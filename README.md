# Cavatrix 🎵🌧️

A high-performance audio-reactive terminal visualizer written in **Rust**, combining the iconic digital rain of **cmatrix** with the real-time frequency analysis of **CAVA**.

The falling code streams dynamically change colors, acceleration, tail luminescence, and character glyphs based on the live audio spectrum.

---

## ✨ Features

- **Live Audio Reactivity via CAVA**: Seamlessly connects to CAVA's raw PCM/FFT audio output over PulseAudio, PipeWire, or ALSA.
- **Stereo & Mono Audio Mapping**:
  - **Stereo Mode**: Bass frequencies pulse at the center of the terminal, treble and highs ripple toward the outer edges.
  - **Mono Mode**: Bass on the left progressing to highs on the right.
- **Audio-Reactive Color Palettes**:
  - **Matrix Supernova**: Classic phosphor green when calm $\rightarrow$ flaring into electric lime, cyan, blazing amber, and hot supernova pink on heavy beat drops.
  - **Cyberpunk Synthwave**: Deep midnight indigo $\rightarrow$ electric violet $\rightarrow$ neon cyan $\rightarrow$ hot magenta pink.
  - **Rainbow Spectrum**: Full-spectrum chromatic wave modulated by frequency and sound amplitude.
  - **Ice & Fire**: Cold glacial arctic cyan igniting into flaming gold and lava on loud beats.
  - **Amane Sunset**: Twilight velvet fading into coral salmon (`#f56d66`) and sunrise gold.
- **Integrated CAVA Equalizer Baseline**: Optional real-time spectrum bar baseline rendered directly beneath the matrix streams.
- **Sub-character Smoothness**: Double-buffered 60 FPS TrueColor terminal rendering with zero flicker.
- **Graceful Fallback**: Includes an ambient harmonic audio generator if CAVA is silent or temporarily inactive.

---

## ⌨️ Controls

| Key | Action |
|---|---|
| `q` or `Esc` | Exit `cavatrix` cleanly |
| `c` | Cycle color palette mode |
| `m` | Toggle Stereo (center bass) vs Mono (left-to-right) mapping |
| `b` | Toggle bottom CAVA equalizer baseline wave |
| `+` / `=` | Increase audio sensitivity / gain |
| `-` / `_` | Decrease audio sensitivity / gain |
| `]` | Increase rain speed |
| `[` | Decrease rain speed |
| `Space` | Pause / Resume animation |
| `h` or `?` | Toggle help legend overlay |

---

## 🚀 Installation & Running

The binary is compiled and installed to `~/.local/bin/cavatrix`.

Run directly in any terminal:
```bash
cavatrix
```
