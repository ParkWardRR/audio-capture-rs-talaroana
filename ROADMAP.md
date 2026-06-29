# Cross-Platform Audio Capture Engineering Roadmap

This document outlines the strategic vision and engineering milestones for the project. Our objective is to deliver the industry's most performant, secure, and robust audio capture implementation, prioritizing **Rust for high-performance core logic**, strict real-time safety, and ensuring seamless integration capabilities.

## Phase 1: Baseline Native Integration (Completed)
**Focus:** API boundary definition, cross-platform architecture, and build infrastructure.

- [x] Established `AudioCaptureBackend` API trait as the core architectural boundary.
- [x] **macOS:** Initialized AudioUnit graph (`kAudioUnitSubType_HALOutput`).
- [x] **Linux:** Implemented subprocess-based fallback capture mechanisms.
- [x] Set up comprehensive local CI pipeline structures.
- [x] Scaffolded advanced fuzzing targets for boundary condition coverage.

## Phase 2: Full Implementation & Real-Time Extreme Performance (In Progress)
**Focus:** Finalizing native capture backends, real-time safety, zero-copy architectures, and robust fuzzing.

- [ ] **macOS Native Callback:** Implement `AURenderCallbackStruct` to efficiently stream interleaved PCM data.
- [ ] **Linux Native Bindings:** Transition to native `libpipewire` bindings for the absolute lowest possible latency, bypassing subprocesses.
- [ ] **Arch Linux / SteamOS Support:** Optimize PipeWire integration for native Steam Deck compatibility and gaming-oriented low latency.
- [ ] **Real-Time Safety Validation:** Enforce `#![deny(clippy::alloc_instead_of_core)]` and strict architectural constraints to ensure the audio callback never allocates, locks, or blocks on I/O.
- [ ] **Lock-Free Concurrency (SPSC):** Introduce single-producer, single-consumer ring buffers (e.g., via `ringbuf`) for deterministic, lock-free data handoffs from the capture thread to the processing pool.
- [ ] **CPU Core Pinning:** Add capabilities to isolate and pin high-priority audio threads to specific CPU cores, eliminating context-switch jitter.
- [ ] **Robust Fuzzing Integration:** Expand fuzzing logic to thoroughly validate audio pipeline boundaries.
- [ ] **SIMD Optimization Expansion:** Offload compute-heavy paths to explicit NEON (ARM64) and AVX-512 (x86_64) intrinsic implementations.
- [ ] **`#![no_std]` Compliance:** Introduce a comprehensive `no_std` feature flag for bare-metal microcontroller execution.
- [ ] **Zero-Copy Pipeline Architecture:** Implement a unified zero-allocation pipeline to completely eliminate intermediate buffer allocations.

## Phase 3: High-Throughput Orchestration & Streaming
**Focus:** Seamless integration of the ultra-fast Rust core into large-scale microservices and peer-to-peer networks.

- [x] **Idiomatic Foreign Function Interfaces:** Developed safe, zero-allocation modules bridging the Rust FFI boundary.
- [ ] **WebRTC Media Streaming:** Implement direct WebRTC Data Channel and Media Stream wrappers to enable sub-500ms global broadcast of raw audio frames.
- [ ] **Streaming Interface Implementations:** Ensure compatibility with standard I/O ecosystems via memory-mapped buffers.
- [ ] **Cloud-Native Orchestration:** Build distributed encoding pipelines capable of managing pools of hardware-accelerated worker nodes.
- [ ] **Cross-Language CI:** Expand testing frameworks to continuously verify bit-perfect equivalence across FFI boundaries.

## Phase 4: Advanced Rust Ecosystem Integrations
**Focus:** Leveraging cutting-edge frameworks for high-throughput deployment.

- [ ] **`tokio-uring` / `io_uring` Support:** Exploit Linux's `io_uring` via async runtimes for zero-copy file and network I/O.
- [ ] **Distributed Execution via `tonic` (gRPC):** Develop microservice scaffolding to scale horizontally across Kubernetes clusters.
- [ ] **eBPF Tracing Hooks:** Embed USDT probes directly into the core for advanced latency profiling in production environments without overhead.
- [ ] **JACK Audio Connection Kit:** Add native JACK bindings to support professional Linux audio environments alongside PipeWire.

## Phase 5: Future-Proofing & System Integration
**Focus:** Broadening the scope of the project beyond conventional architectures.

- [ ] **Hardware-Accelerated Security:** Combine execution with encryption to provide secure streams for enterprise use cases.
- [ ] **WebAssembly (WASM) Module:** Ensure compilation to `wasm32-unknown-unknown` with web-workers support.
- [ ] **Custom Hardware DSP Targets:** Explore compilation and deployment patterns for specialized Digital Signal Processors and FPGAs.
- [ ] **Direct DMA Audio Capture:** Investigate direct memory access capture techniques bypassing OS audio subsystems for ultra-low microsecond latency.
- [ ] **Windows ASIO Integration:** Implement direct ASIO driver interactions for professional, low-latency audio capture on Windows.
