use crate::config::VoiceConfig;
use crate::error::{AppError, Result};
use crate::io::InputSource;
use async_trait::async_trait;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleRate, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const WHISPER_SAMPLE_RATE: u32 = 16000;
const CHUNK_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
enum VoiceState {
    WaitingForVoice,
    Recording,
    SilenceDetected { silence_sample: usize },
}

pub struct VoiceInput {
    whisper_ctx: WhisperContext,
    device: Device,
    config: VoiceConfig,
    device_sample_rate: u32,
    stop_signal: Arc<AtomicBool>,
}

impl VoiceInput {
    pub fn new(config: &VoiceConfig) -> Result<Self> {
        tracing::info!("init voice model, {}", config.model_path);

        let whisper_ctx = Self::init_whisper(&config.model_path)?;

        let (device, device_sample_rate) = Self::init_audio_device()?;

        Ok(Self {
            whisper_ctx,
            device,
            config: config.clone(),
            device_sample_rate,
            stop_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    fn init_whisper(model_path: &str) -> Result<WhisperContext> {
        tracing::debug!("loading Whisper model: {}", model_path);

        if !std::path::Path::new(model_path).exists() {
            return Err(AppError::SpeechRecognition(format!(
                "model is not exist: {}",
                model_path
            )));
        }

        let ctx_params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(model_path, ctx_params).map_err(|e| {
            AppError::speech_recognition(format!("load Whisper model failed: {}", e))
        })?;

        tracing::info!("Whisper model load successed!");
        Ok(ctx)
    }

    fn init_audio_device() -> Result<(Device, u32)> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or(AppError::NoAudioDevice)?;
        let device_name = device
            .name()
            .unwrap_or_else(|_| "unknown device".to_string());
        tracing::info!("using audio devie: {}", device_name);

        let supported_config = device
            .default_input_config()
            .map_err(|e| AppError::audio(format!("get audio config failed: {}", e)))?;

        let sample_rate = supported_config.sample_rate().0;
        tracing::debug!(
            "sample rate: {} Hz, format: {:?}",
            sample_rate,
            supported_config.sample_format()
        );

        Ok((device, sample_rate))
    }

    async fn record_audio(&self) -> Result<Vec<f32>> {
        let (tx, mut rx) = mpsc::channel::<Vec<f32>>(64);
        let stop_signal = self.stop_signal.clone();

        let stream_config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(self.device_sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let device_sample_rate = self.device_sample_rate;

        let stream = self.build_audio_stream(&stream_config, tx.clone())?;

        stream
            .play()
            .map_err(|e| AppError::audio(format!("start recorading failed: {}", e)))?;

        tracing::info!("start recording, please say something...");

        let mut audio_buffer = Vec::new();
        let mut state = VoiceState::WaitingForVoice;

        let silence_threshold_sample =
            (self.config.silience_threshold_secs * device_sample_rate as f32) as usize;
        let max_samples = (self.config.max_duration_secs * device_sample_rate as f32) as usize;

        const ENERGY_THRESHOLD: f32 = 0.01;

        while let Some(chunk) = rx.recv().await {
            if stop_signal.load(Ordering::Relaxed) {
                tracing::debug!("recv the stop signal");
                break;
            }

            let energy = Self::calculate_energy(&chunk);
            let has_voice = energy > ENERGY_THRESHOLD;

            state = match state {
                VoiceState::WaitingForVoice => {
                    if has_voice {
                        tracing::debug!("detect voice, energry: {:.4}", energy);
                        audio_buffer.extend_from_slice(&chunk);
                        VoiceState::Recording
                    } else {
                        VoiceState::WaitingForVoice
                    }
                }
                VoiceState::Recording => {
                    audio_buffer.extend_from_slice(&chunk);

                    if !has_voice {
                        VoiceState::SilenceDetected {
                            silence_sample: chunk.len(),
                        }
                    } else {
                        VoiceState::Recording
                    }
                }
                VoiceState::SilenceDetected { silence_sample } => {
                    audio_buffer.extend_from_slice(&chunk);

                    if has_voice {
                        VoiceState::Recording
                    } else {
                        let new_silence = silence_sample + chunk.len();
                        if new_silence >= silence_threshold_sample {
                            tracing::debug!("detect voice stopped, silence {} sample", new_silence);
                            break;
                        }
                        VoiceState::SilenceDetected {
                            silence_sample: new_silence,
                        }
                    }
                }
            };

            if audio_buffer.len() >= max_samples {
                tracing::warn!("reach max recording time");
                break;
            }
        }

        drop(stream);

        if audio_buffer.is_empty() {
            return Err(AppError::audio("no audio signal"));
        }

        tracing::debug!("recording successed!, sample point {}", audio_buffer.len());

        let resampled = self.resample_audio(&audio_buffer)?;

        Ok(resampled)
    }

    fn build_audio_stream(
        &self,
        config: &StreamConfig,
        tx: mpsc::Sender<Vec<f32>>,
    ) -> Result<Stream> {
        let err_fn = |err| tracing::error!("audio stream error: {}", err);

        let stream = self
            .device
            .build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let _ = tx.try_send(data.to_vec());
                },
                err_fn,
                None,
            )
            .map_err(|e| AppError::audio(format!("create audio stram failed: {}", e)))?;

        Ok(stream)
    }

    fn calculate_energy(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    fn resample_audio(&self, audio: &[f32]) -> Result<Vec<f32>> {
        if self.device_sample_rate == WHISPER_SAMPLE_RATE {
            return Ok(audio.to_vec());
        }

        use rubato::{FftFixedInOut, Resampler};

        let resample_ratio = WHISPER_SAMPLE_RATE as f64 / self.device_sample_rate as f64;

        let mut resampler = FftFixedInOut::<f32>::new(
            self.device_sample_rate as usize,
            WHISPER_SAMPLE_RATE as usize,
            1024,
            1,
        )
        .map_err(|e| AppError::audio(format!("create resampler failed: {}", e)))?;

        let input_frames = resampler.input_frames_next();
        let mut output = Vec::new();

        for chunk in audio.chunks(input_frames) {
            if chunk.len() < input_frames {
                let mut padded = chunk.to_vec();
                padded.resize(input_frames, 0.0);
                let result = resampler
                    .process(&[padded], None)
                    .map_err(|e| AppError::audio(format!("resample failed: {}", e)))?;

                output.extend_from_slice(&result[0]);
            } else {
                let result = resampler
                    .process(&[chunk.to_vec()], None)
                    .map_err(|e| AppError::audio(format!("resample failed: {}", e)))?;
                output.extend_from_slice(&result[0]);
            }
        }

        let expected_len = (audio.len() as f64 * resample_ratio) as usize;
        output.truncate(expected_len);

        tracing::debug!(
            "resampled successed: {} -> {} sample point",
            audio.len(),
            output.len()
        );

        Ok(output)
    }

    fn transcribe(&self, audio: &[f32]) -> Result<String> {
        tracing::debug!(
            "start voice transcrining, audio len: {} points",
            audio.len()
        );

        let mut state = self.whisper_ctx.create_state().map_err(|e| {
            AppError::speech_recognition(format!("create Whisper status failed: {}", e))
        })?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        if self.config.language != "auto" {
            params.set_language(Some(&self.config.language));
        }

        params.set_translate(self.config.translate);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_single_segment(true);

        state.full(params, audio).map_err(|e| {
            AppError::speech_recognition(format!("get transcribe result failed: {}", e))
        })?;

        let num_segments = state.full_n_segments().map_err(|e| {
            AppError::speech_recognition(format!("get segments count failed: {}", e))
        })?;

        let mut result = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                result.push_str(&segment);
            }
        }

        let trimmed = result.trim().to_string();
        tracing::info!("transcribe result: {}", trimmed);

        Ok(trimmed)
    }

    pub fn stop(&self) {
        self.stop_signal.store(true, Ordering::Relaxed);
    }
}

#[async_trait]
impl InputSource for VoiceInput {
    async fn next(&mut self) -> Result<Option<String>> {
        self.stop_signal.store(false, Ordering::Relaxed);

        let audio = match self.record_audio().await {
            Ok(audio) => audio,
            Err(e) if matches!(e, AppError::Cancelled) => {
                return Ok(None);
            }
            Err(e) => {
                tracing::error!("recording failed: {}", e);
                return Err(e);
            }
        };

        let text = self.transcribe(&audio)?;

        if text.is_empty() {
            tracing::debug!("transcribe result is empty, waiting...");
            return self.next().await;
        }

        Ok(Some(text))
    }
}

impl Drop for VoiceInput {
    fn drop(&mut self) {
        self.stop();
        tracing::debug!("voice input has been freed");
    }
}
