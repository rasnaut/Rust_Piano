# SensitHaptics - Software Engineer Assignment

## Overview

Welcome to the SensitHaptics technical assignment! This exercise evaluates your ability to work with real-time audio systems, multithreading, and data flow between Rust and a React frontend.

You'll be working on a piano synthesizer application built with **Tauri** (Rust backend + React frontend). The application already generates audio - your task is to implement **real-time waveform visualization**.

---

## The Challenge

Implement a real-time waveform visualizer that displays the audio output as it plays.

### What You Need to Build

#### Rust Backend (`src-tauri/src/lib.rs`)

1. **Ring Buffer**: Implement a thread-safe ring buffer that stores recent audio samples
   - The audio callback runs at ~48,000 samples/second
   - Store enough samples for visualization 
   - Must be lock-free or use minimal locking to avoid audio glitches

2. **IPC Command**: Implement the `get_waveform_data` command
   - Returns a snapshot of recent audio samples to the frontend
   - Should return a `Vec<f32>` of sample values

#### What's Already Provided

- Working piano synthesizer with polyphonic audio
- React UI with a canvas component ready for visualization
- The `get_waveform_data` IPC command stub
- Canvas drawing setup and animation loop

### Requirements

1. **No audio glitches**: The visualization must not introduce latency or audio dropouts
2. **Smooth visualization**: The waveform should update smoothly (~30-60 fps)
3. **Thread safety**: Proper synchronization between audio thread and IPC calls
4. **Clean code**: Follow Rust best practices, handle errors appropriately

---

## Getting Started

### Prerequisites

- Node.js 18+ and pnpm
- Rust 1.70+
- Windows (primary platform)

### Setup

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Project Structure

```
src/                      # React Frontend
  App.tsx                 # Piano UI + Waveform visualizer
  App.css                 # Styling

src-tauri/                # Rust Backend
  src/
    lib.rs                # Synthesizer + Audio engine (YOUR WORK HERE)
  Cargo.toml              # Dependencies
```

---

## Submission

1. Ensure `npm run tauri dev` runs without errors
2. Test that you can play the piano while seeing the waveform
3. Commit your changes with clear commit messages
4. Push to your repository or send us a zip file

---

## Questions?

If you have questions about the assignment, please reach out to us. We're happy to clarify requirements - we want to see your engineering skills, not test your mind-reading abilities!

Good luck!
