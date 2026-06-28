# Rust Audio Capture Engine Roadmap

## Completed (Prior History)
- [x] Implemented backend instantiation and stream capturing logic.
- [x] Added `cargo-fuzz` and `criterion` benchmarks for audio buffer throughput.
- [x] Fixed all Rust 1.93 pedantic lints and formatting deviations.
- [x] Setup robust CI pipelines.

## Short-term Goals
- [ ] Expand macOS CoreAudio and Linux ALSA specific backend features (loopback capture).
- [ ] Ensure 100% thread-safe ring-buffer integration out-of-the-box.
- [ ] Stabilize device enumeration APIs.

## Mid-term Goals
- [ ] Add WASAPI loopback support for Windows environments.
- [ ] Implement automatic drift correction across independent input interfaces.
- [ ] Provide C FFI for integration into existing legacy systems.

## Long-term Vision
- [ ] Create a comprehensive universal audio I/O abstraction layer rivaling CPAL, with a focus purely on low-latency capture.
