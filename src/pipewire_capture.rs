//! Linux PipeWire capture backend — native implementation.
//!
//! Uses `pw-cat --record` subprocess for initial implementation (reliable,
//! zero external crate deps, works with any PipeWire version).
//!
//! Architecture:
//! 1. Spawns `pw-cat --record` with format/rate/channels matching CaptureConfig
//! 2. Reads raw PCM from stdout in a dedicated thread
//! 3. Delivers fixed-size frames (352 samples = ALAC frame) to the callback
//! 4. Falls back to `pacat --record` if PipeWire is unavailable
//!
//! This approach provides:
//! - Zero-dependency capture (no pipewire-rs crate, no C FFI)
//! - Works with PipeWire's PulseAudio compatibility layer too
//! - Correct sample format negotiation via command-line flags
//! - Low latency: pw-cat uses the PipeWire graph clock natively

use crate::{AudioCaptureBackend, CaptureCallback, CaptureConfig, CaptureSource, SampleFormat};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// PipeWire capture backend using `pw-cat` subprocess.
pub struct PipeWireCapture {
    child: Option<Child>,
    running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl PipeWireCapture {
    pub fn new() -> Self {
        Self {
            child: None,
            running: Arc::new(AtomicBool::new(false)),
            thread: None,
        }
    }

    /// Check if pw-cat is available.
    fn has_pw_cat() -> bool {
        Command::new("pw-cat").arg("--version").output().is_ok()
    }

    /// Check if pacat (PulseAudio) is available.
    fn has_pacat() -> bool {
        Command::new("pacat").arg("--version").output().is_ok()
    }

    /// Detect the default monitor source for PulseAudio/PipeWire.
    fn detect_monitor_source() -> Option<String> {
        let output = Command::new("pactl")
            .args(["get-default-sink"])
            .output()
            .ok()?;
        let sink = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if sink.is_empty() {
            return None;
        }
        Some(format!("{}.monitor", sink))
    }

    fn format_to_pw_format(format: SampleFormat) -> &'static str {
        match format {
            SampleFormat::S16LE => "s16",
            SampleFormat::S24LE => "s24",
            SampleFormat::S32LE => "s32",
            SampleFormat::F32LE => "f32",
        }
    }

    fn format_to_pa_format(format: SampleFormat) -> &'static str {
        match format {
            SampleFormat::S16LE => "s16le",
            SampleFormat::S24LE => "s24le",
            SampleFormat::S32LE => "s32le",
            SampleFormat::F32LE => "float32le",
        }
    }

    fn bytes_per_sample(format: SampleFormat) -> usize {
        match format {
            SampleFormat::S16LE => 2,
            SampleFormat::S24LE => 3,
            SampleFormat::S32LE => 4,
            SampleFormat::F32LE => 4,
        }
    }
}

impl AudioCaptureBackend for PipeWireCapture {
    fn start(
        &mut self,
        config: CaptureConfig,
        mut callback: CaptureCallback,
    ) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Err("capture already running".into());
        }

        let frame_bytes = config.buffer_frames as usize
            * config.channels as usize
            * Self::bytes_per_sample(config.format);

        // Try pw-cat first, then pacat.
        let mut child = if Self::has_pw_cat() {
            let pw_format = Self::format_to_pw_format(config.format);
            Command::new("pw-cat")
                .args([
                    "--record",
                    "--format",
                    pw_format,
                    "--rate",
                    &config.sample_rate.to_string(),
                    "--channels",
                    &config.channels.to_string(),
                    "--quality",
                    "0", // disable resampling
                    "-", // output to stdout
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("failed to start pw-cat: {}", e))?
        } else if Self::has_pacat() {
            let pa_format = Self::format_to_pa_format(config.format);
            let monitor =
                Self::detect_monitor_source().ok_or("no PulseAudio monitor source found")?;

            Command::new("pacat")
                .args([
                    "--record",
                    "--format",
                    pa_format,
                    "--rate",
                    &config.sample_rate.to_string(),
                    "--channels",
                    &config.channels.to_string(),
                    "--device",
                    &monitor,
                    "--raw",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("failed to start pacat: {}", e))?
        } else {
            return Err("neither pw-cat nor pacat found — install PipeWire or PulseAudio".into());
        };

        let mut stdout = child
            .stdout
            .take()
            .ok_or("failed to capture stdout from audio process")?;

        self.running.store(true, Ordering::Release);
        let running = self.running.clone();

        self.thread = Some(thread::spawn(move || {
            let mut buf = vec![0u8; frame_bytes];

            while running.load(Ordering::Acquire) {
                match stdout.read_exact(&mut buf) {
                    Ok(()) => {
                        callback(&buf);
                    }
                    Err(e) => {
                        if running.load(Ordering::Acquire) {
                            eprintln!("[pipewire] read error: {}", e);
                        }
                        break;
                    }
                }
            }
        }));

        self.child = Some(child);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        self.running.store(false, Ordering::Release);

        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }

        Ok(())
    }

    fn list_sources(&self) -> Vec<CaptureSource> {
        // Use pactl to enumerate sinks and their monitors.
        let output = match Command::new("pactl")
            .args(["list", "short", "sinks"])
            .output()
        {
            Ok(o) => o,
            Err(_) => return vec![],
        };

        let text = String::from_utf8_lossy(&output.stdout);
        let mut sources = Vec::new();

        for line in text.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let name = parts[1].to_string();
                sources.push(CaptureSource {
                    id: format!("{}.monitor", name),
                    name: format!("{} (monitor)", name),
                    is_loopback: true,
                    sample_rate: 0, // detected at runtime
                    channels: 0,
                });
            }
        }

        sources
    }
}
