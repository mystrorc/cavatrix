# Cavatrix 🎵🌧️

A high-performance audio-reactive terminal visualizer written in **Rust**, combining the iconic digital rain of **cmatrix** with the real-time frequency analysis of **CAVA**.

The falling code streams dynamically change colors, acceleration, tail luminescence, and character glyphs based on the live audio spectrum, raining straight through multi-row CAVA equalizer bars that rise and bounce past half the screen.

---

## ✨ Features

- **Multi-Row CAVA Equalizer Bars**:
  - Bars rise up from the bottom up to **65%–90%** of the screen height based on real-time audio energy.
  - Floating peak caps (`▔`) with gravitational dropoff.
  - Sub-character smooth peak resolution (` `, `▂`, `▃`, `▄`, `▅`, `▆`, `▇`).
- **Full-Depth Digital Rain**:
  - Matrix glyphs (half-width Katakana, numbers, and symbols) rain continuously from the ceiling all the way down to the bottom floor.
  - Glyphs passing through active CAVA bars pop with heightened electric luminosity and high-contrast punch.
- **Ultra Low-CPU Architecture**:
  - Zero-allocation double-buffered flat rendering pipeline.
  - Sequential cursor tracking eliminating ANSI escape coordinate floods.
  - Configurable 30 FPS default for lightweight background operation with 60 FPS support on demand.
- **Bar Styles (`v` key)**:
  - **Solid** (`█`): Classic solid equalizer columns.
  - **Shaded** (`▒`): Semi-translucent digital holographic grid.
  - **Dense** (`▓`): High-density textured bars.
  - **Pillar** (`┃`): Thin equalizer needles.
- **Audio-Reactive Color Palettes (`c` key)**:
  - **Matrix Supernova**: Classic phosphor green $\rightarrow$ electric lime $\rightarrow$ cyan $\rightarrow$ blazing amber $\rightarrow$ hot supernova pink (`#ff0055`).
  - **Cyberpunk Synthwave**: Velvet purple $\rightarrow$ electric violet $\rightarrow$ neon cyan $\rightarrow$ hot magenta pink.
  - **Rainbow Spectrum**: Full chromatic wave across columns and frequencies.
  - **Ice & Fire**: Glacial arctic cyan igniting into flaming gold and molten lava.
  - **Amane Sunset**: Soft twilight violet blending into desktop coral (`#f56d66`) and sunrise gold.
- **Stereo & Mono Audio Mapping (`m` key)**:
  - **Stereo**: Bass in the center, treble spreading to both sides.
  - **Mono**: Bass on left progressing to highs on the right.

---

## 🚀 Running & CLI Options

Run directly in any terminal:
```bash
cavatrix
```

### Command-Line Flags
```bash
cavatrix --fps 60        # Run at 60 FPS (default: 30 FPS)
cavatrix --mono          # Start in mono audio mapping
cavatrix --bars 48       # Set 48 frequency spectrum bars
cavatrix --speed 1.5     # Faster rain falling speed
cavatrix --help          # Show full help information
```

```text
USAGE:
    cavatrix [OPTIONS]

OPTIONS:
    -f, --fps <FPS>       Target framerate in frames per second [default: 30] (e.g. 30, 60)
    -b, --bars <BARS>     Number of audio frequency bars [default: 64]
    -m, --mono            Start in mono audio mode (Bass on left -> Treble on right)
    -s, --speed <SPEED>   Initial rain falling speed multiplier [default: 1.0]
    -h, --help            Print help information
    -v, --version         Print version information
```

---

## ⌨️ Controls

| Key | Action |
|---|---|
| `q` or <kbd>Esc</kbd> | Exit `cavatrix` cleanly |
| `c` | Cycle color palette |
| `v` | Cycle bar style (Solid `█`, Shaded `▒`, Dense `▓`, Pillar `┃`) |
| `b` | Toggle CAVA equalizer bars ON/OFF |
| <kbd>Up</kbd> / <kbd>Down</kbd> | Adjust max bar height (from 20% up to 95% of screen) |
| `m` | Toggle Stereo (center bass) vs Mono (left-to-right) |
| `+` / `=` | Increase audio sensitivity / gain |
| `-` / `_` | Decrease audio sensitivity / gain |
| `]` / `[` | Speed up / slow down rain falling speed |
| <kbd>Space</kbd> | Pause / Resume animation |
| `h` or `?` | Toggle help legend overlay |

---

## 📦 Installation & Building

```bash
git clone https://github.com/mystrorc/cavatrix.git
cd cavatrix
cargo build --release
cp target/release/cavatrix ~/.local/bin/
```
