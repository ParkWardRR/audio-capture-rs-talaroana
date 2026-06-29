# Cross-Platform Audio Capture Engineering Roadmap

This document outlines the strategic vision and engineering milestones for the `audio-capture-rs-talaroana` project. Our objective is to deliver the most performant, secure, and robust implementation, prioritizing **Rust for high-performance core logic**, leveraging **tons of hardware acceleration**, and utilizing **Go for versatile cross-language tooling and systems integration**.

## Phase 1: Core Stabilization & Ergonomics (Completed)
**Focus:** API stabilization, basic functionality, and comprehensive error handling.

- [x] Implemented core algorithms and baseline validation.
- [x] Integrated `fuzzing` targets and achieved robust boundary condition coverage.
- [x] Established strict local CI testing pipelines via OrbStack + Act.
- [x] v1.0.0 API Freeze for downstream consumers.

## Phase 2: Extreme Performance & Hardware Acceleration (Rust)
**Focus:** Maximizing throughput via hardware acceleration, zero-copy pipelines, and advanced instruction sets.

- [ ] **SIMD Optimization Expansion:** Offload compute-heavy paths (like sample rate conversion and format packing) to explicit NEON (ARM64) and AVX-512 (x86_64) intrinsic implementations.
- [ ] **GPU-Accelerated Audio Processing:** Integrate compute shaders (via `wgpu` or Vulkan) to offload large batch DSP operations directly to the GPU.
- [ ] **Apple Neural Engine (ANE) & Tensor Core Offloading:** Implement real-time, zero-latency noise suppression and echo cancellation utilizing hardware AI accelerators.
- [ ] **Hardware AES-NI Encryption:** Combine execution with AES-NI instructions to provide secure, encrypted streams for enterprise use cases with zero CPU overhead.
- [ ] **Zero-Copy Pipeline Architecture:** Implement a unified zero-allocation pipeline leveraging memory-mapped I/O to completely eliminate intermediate buffer allocations.

## Phase 3: Go Integration & High-Throughput Orchestration
**Focus:** Bringing highly optimized Rust core logic to Go-based microservices and infrastructure.

- [ ] **Idiomatic Go Wrapper (`cgo` bindings):** Develop a safe, zero-allocation Go module bridging the Rust FFI boundary, passing memory pointers efficiently.
- [ ] **Go Interface Implementations:** Implement streaming interfaces seamlessly compatible with the Go standard library's `io.Reader` and `io.Writer` ecosystem.
- [ ] **Cloud-Native Go Orchestrator:** Build reference distributed pipelines in Go that manage pools of hardware-accelerated Rust worker nodes.
- [ ] **gRPC / ConnectRPC Streaming Servers:** Implement high-throughput, low-latency audio streaming servers in Go to broadcast the captured data.
- [ ] **Cross-Language CI:** Expand testing to run Go-Rust integration tests and fuzzing across the FFI boundary.

## Phase 4: Advanced Rust Ecosystem Integrations
**Focus:** Leveraging cutting-edge frameworks for high-throughput deployment.

- [ ] **`tokio-uring` / `io_uring` Support:** Exploit Linux's `io_uring` via async runtimes for zero-copy file and network I/O.
- [ ] **eBPF Tracing Hooks:** Embed USDT probes directly into the Rust core for advanced latency profiling in production environments without overhead.
- [ ] **`#![no_std]` Compliance:** Introduce a comprehensive `no_std` feature flag for bare-metal microcontroller execution.

## Phase 5: Future-Proofing & Specialized Hardware
**Focus:** Broadening the scope of the project beyond simple execution and generic CPUs.

- [ ] **Custom Hardware DSP Targets:** Explore compilation and deployment patterns for specialized Digital Signal Processors and FPGAs.
- [ ] **WebAssembly (WASM) Module:** Ensure compilation to `wasm32-unknown-unknown` with web-workers and WebGPU support for in-browser hardware acceleration.
- [ ] **Direct DMA Audio Capture:** Investigate direct memory access capture techniques bypassing OS audio subsystems for ultra-low microsecond latency.
