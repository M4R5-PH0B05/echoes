# 🎧 echoes

> A real-time audio visualizer for your terminal — built in Rust.

Echoes turns your terminal into a pulsing soundscape.  
It reads audio files (MP3, WAV, FLAC, etc.) and renders live waveforms or frequency spectrums directly in the terminal, in real time.  

---

## Features

- Decode and visualize **MP3, WAV, FLAC** and more (via [Symphonia](https://crates.io/crates/symphonia))  
- Real-time waveform and spectrum modes  
- Optional color gradients 
- Fully offline — no internet or API keys  
- Works on macOS, Linux, and Windows  
- Lightweight (under 3MB binary)

---

## Installation

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable recommended)

### Build from source
```bash
git clone https://github.com/yourname/echoes.git
cd echoes
cargo build --release
