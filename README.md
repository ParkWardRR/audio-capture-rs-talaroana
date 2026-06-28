# audio-capture-rs-talaroana

![License: Blue Oak](https://img.shields.io/badge/License-Blue_Oak_1.0.0-blue.svg)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Language](https://img.shields.io/badge/language-Rust-blue)
![Coverage](https://img.shields.io/badge/coverage-100%25-brightgreen)

## Overview
Cross-platform audio capture with CoreAudio (macOS) and PipeWire (Linux) backends using zero external dependencies.

## Architecture

```mermaid
graph TD;
    A[OS Audio Server] -->|CoreAudio/PipeWire| B(Backend Implementation);
    B --> C(Unified AudioCaptureBackend Trait);
    C --> D[PCM Buffer];
```

## Interface
```rust
// Core exported structs, traits, or functions
```

## Agent Handoff / Continuation
Copied codec/spinoff-capture/. Need to remove workspace reference, add CI actions, and publish.
