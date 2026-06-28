//! Cross-platform audio capture for spinoff.
//!
//! Platform backends:
//! - macOS: CoreAudio AudioUnit (ScreenCaptureKit audio tap on macOS 14.4+)
//! - Linux: PipeWire native client (fallback: PulseAudio via libpulse)
//!
//! The unified `AudioCapture` trait provides a platform-agnostic interface
//! for Go to call via C FFI.

#[cfg(target_os = "macos")]
pub mod coreaudio;

#[cfg(target_os = "linux")]
pub mod pipewire_capture;

/// Audio sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum SampleFormat {
    S16LE,  // 16-bit signed little-endian
    S24LE,  // 24-bit signed little-endian (packed)
    S32LE,  // 32-bit signed little-endian
    F32LE,  // 32-bit float little-endian
}

/// Audio capture configuration.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct CaptureConfig {
    pub sample_rate: u32,
    pub channels: u32,
    pub format: SampleFormat,
    pub buffer_frames: u32, // frames per callback
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            format: SampleFormat::S16LE,
            buffer_frames: 352, // ALAC frame size
        }
    }
}

/// Callback type: receives interleaved PCM samples.
pub type CaptureCallback = Box<dyn FnMut(&[u8]) + Send + 'static>;

/// Platform-agnostic audio capture trait.
pub trait AudioCaptureBackend: Send {
    /// Start capturing audio. The callback receives interleaved PCM data.
    fn start(&mut self, config: CaptureConfig, callback: CaptureCallback) -> Result<(), String>;

    /// Stop capturing.
    fn stop(&mut self) -> Result<(), String>;

    /// List available capture sources.
    fn list_sources(&self) -> Vec<CaptureSource>;
}

/// Describes an available audio capture source.
#[derive(Debug, Clone)]
pub struct CaptureSource {
    pub id: String,
    pub name: String,
    pub is_loopback: bool,
    pub sample_rate: u32,
    pub channels: u32,
}

/// Create the platform-appropriate capture backend.
pub fn create_backend() -> Box<dyn AudioCaptureBackend> {
    #[cfg(target_os = "macos")]
    {
        Box::new(coreaudio::CoreAudioCapture::new())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(pipewire_capture::PipeWireCapture::new())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        compile_error!("Unsupported platform — need macOS or Linux");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 2);
        assert_eq!(config.format, SampleFormat::S16LE);
        assert_eq!(config.buffer_frames, 352);
    }
    
    // We cannot reliably test CoreAudio in CI without hardware access,
    // so we just test that the trait is object-safe and backend creates.
    #[test]
    fn test_create_backend() {
        let mut backend = create_backend();
        let sources = backend.list_sources();
        // Just ensuring it doesn't crash. Sources might be empty in CI.
        let _ = sources.len();
    }
}
