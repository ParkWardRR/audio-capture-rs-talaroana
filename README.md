# Cross-Platform Audio Capture

![Language](https://img.shields.io/badge/Language-Rust-blue.svg)
![License](https://img.shields.io/badge/License-BlueOak_1.0.0-green.svg)
![Status](https://img.shields.io/badge/Status-Production_Ready-brightgreen.svg)

## Overview
Zero-dependency Rust library for system-level audio capture. Uses raw CoreAudio (macOS) and PipeWire (Linux) backends for ultra-low latency.

Designed strictly for high-performance integrations and infrastructure codebases. No redundant abstractions; focuses entirely on precise data processing.

## Architecture

```mermaid
graph TD;
    A[CoreAudio / PipeWire] --> B[Capture Callback];
    B --> C[Interleaved PCM Buffer];

```

## Requirements
- **Rust**: Latest stable toolchain.
- **OS Support**: Cross-platform (macOS/Linux prioritized).
- **Dependencies**: Minimal to none (strictly constrained to standard library where mathematically possible).

## Quick Tutorial

Integration is straightforward. Consult the module source for exact API signatures.

```rust
// 1. Initialize the primary component
// 2. Supply the required I/O interfaces or buffers
// 3. Execute the processing loop or listener
```
*(Refer to the in-code documentation and `*_test.rs` files for exhaustive initialization examples and constraints).*
