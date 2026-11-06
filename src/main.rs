mod audio;
mod color;
mod matrix;

use std::io::{self, BufWriter, Write};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use audio::AudioProcessor;
use color::{ColorRgb, PaletteMode};
use matrix::MatrixEngine;

struct OsdMessage {
    text: String,
    expiry: Instant,
}

fn main() -> io::Result<()> {
    // Install panic hook to ensure terminal state is always restored
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        default_panic(info);
    }));

    enable_raw_mode()?;
    let mut stdout = BufWriter::with_capacity(65536, io::stdout());
    execute!(stdout, EnterAlternateScreen, Hide, crossterm::terminal::SetTitle("cavatrix"))?;

    let (mut cols, mut rows) = crossterm::terminal::size()?;
    let mut engine = MatrixEngine::new(cols as usize, rows as usize);
    let mut audio = AudioProcessor::new(128);

    let mut palette = PaletteMode::MatrixSupernova;
    let mut paused = false;
    let mut show_help = false;
    let mut osd: Option<OsdMessage> = Some(OsdMessage {
        text: "CAVATRIX: Audio-Reactive Matrix Rain (Press 'h' for help)".to_string(),
        expiry: Instant::now() + Duration::from_secs(4),
    });

    let mut last_frame = Instant::now();
    let mut prev_grid: Vec<Vec<(char, ColorRgb)>> = vec![vec![(' ', ColorRgb::BLACK); cols as usize]; rows as usize];

    'main_loop: loop {
        let frame_start = Instant::now();
        let dt = last_frame.elapsed().as_secs_f32().min(0.1);
        last_frame = frame_start;

        // Process keyboard & resize events
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        break 'main_loop;
                    }
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break 'main_loop,
                        KeyCode::Char('c') => {
                            palette = palette.next();
                            osd = Some(OsdMessage {
                                text: format!("Palette: {}", palette.name()),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('m') => {
                            audio.stereo = !audio.stereo;
                            let mode_name = if audio.stereo {
                                "Stereo (Bass in Center)"
                            } else {
                                "Mono (Bass on Left -> Treble Right)"
                            };
                            osd = Some(OsdMessage {
                                text: format!("Audio Mapping: {}", mode_name),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('b') => {
                            engine.show_bottom_wave = !engine.show_bottom_wave;
                            osd = Some(OsdMessage {
                                text: format!(
                                    "Bottom Visualizer Wave: {}",
                                    if engine.show_bottom_wave { "ON" } else { "OFF" }
                                ),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            audio.gain = (audio.gain * 1.2).min(5.0);
                            osd = Some(OsdMessage {
                                text: format!("Audio Gain: {:.1}x", audio.gain),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            audio.gain = (audio.gain / 1.2).max(0.2);
                            osd = Some(OsdMessage {
                                text: format!("Audio Gain: {:.1}x", audio.gain),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char(']') => {
                            engine.speed_multiplier = (engine.speed_multiplier * 1.2).min(3.0);
                            osd = Some(OsdMessage {
                                text: format!("Rain Speed: {:.1}x", engine.speed_multiplier),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('[') => {
                            engine.speed_multiplier = (engine.speed_multiplier / 1.2).max(0.3);
                            osd = Some(OsdMessage {
                                text: format!("Rain Speed: {:.1}x", engine.speed_multiplier),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char(' ') => {
                            paused = !paused;
                            osd = Some(OsdMessage {
                                text: if paused { "Paused".to_string() } else { "Resumed".to_string() },
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('h') | KeyCode::Char('?') => {
                            show_help = !show_help;
                        }
                        _ => {}
                    }
                }
                Event::Resize(new_cols, new_rows) => {
                    cols = new_cols;
                    rows = new_rows;
                    engine.resize(cols as usize, rows as usize);
                    prev_grid = vec![vec![(' ', ColorRgb::BLACK); cols as usize]; rows as usize];
                    let _ = execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
                }
                _ => {}
            }
        }

        // Audio update
        audio.update(dt);

        // Matrix update
        if !paused {
            let width = engine.width;
            engine.update(dt, |col| audio.get_column_energy(col, width));
        }

        // Render matrix grid
        let width = engine.width;
        let mut grid = engine.render_frame(palette, |col| audio.get_column_energy(col, width));

        // Overlay OSD notification if active
        if let Some(ref msg) = osd {
            if Instant::now() < msg.expiry {
                let banner = format!(" [ {} ] ", msg.text);
                let banner_len = banner.chars().count();
                if banner_len < cols as usize && rows > 3 {
                    let start_col = (cols as usize - banner_len) / 2;
                    let target_row = 1;
                    for (i, ch) in banner.chars().enumerate() {
                        grid[target_row][start_col + i] = (ch, ColorRgb::new(255, 255, 255));
                    }
                }
            }
        }

        // Overlay Help Legend if active
        if show_help && rows > 10 && cols > 45 {
            let help_lines = [
                "┌────────────────── CAVATRIX HELP ──────────────────┐",
                "│  q / Esc   - Exit cavatrix                        │",
                "│  c         - Cycle color palette                  │",
                "│  m         - Toggle stereo / mono audio mapping   │",
                "│  b         - Toggle bottom CAVA equalizer wave    │",
                "│  + / -     - Adjust audio gain / sensitivity      │",
                "│  [ / ]     - Adjust rain falling speed            │",
                "│  Space     - Pause / Resume visualizer            │",
                "│  h / ?     - Toggle this help dialog              │",
                "└───────────────────────────────────────────────────┘",
            ];
            let start_row = (rows as usize - help_lines.len()) / 2;
            let start_col = (cols as usize - 53) / 2;
            for (r_idx, line) in help_lines.iter().enumerate() {
                for (c_idx, ch) in line.chars().enumerate() {
                    let row = start_row + r_idx;
                    let col = start_col + c_idx;
                    if row < rows as usize && col < cols as usize {
                        grid[row][col] = (ch, ColorRgb::new(200, 250, 255));
                    }
                }
            }
        }

        // Draw double-buffered diff to stdout
        let mut current_fg: Option<ColorRgb> = None;

        for r in 0..rows as usize {
            for c in 0..cols as usize {
                let (ch, col_rgb) = grid[r][c];
                let prev = prev_grid[r][c];

                if ch != prev.0 || col_rgb != prev.1 {
                    let _ = execute!(stdout, MoveTo(c as u16, r as u16));

                    if current_fg != Some(col_rgb) {
                        let _ = write!(
                            stdout,
                            "\x1b[38;2;{};{};{}m",
                            col_rgb.r, col_rgb.g, col_rgb.b
                        );
                        current_fg = Some(col_rgb);
                    }

                    let _ = write!(stdout, "{}", ch);
                }
            }
        }

        let _ = stdout.flush();
        prev_grid = grid;

        // Maintain ~60 FPS
        let elapsed = frame_start.elapsed();
        let target_frame_time = Duration::from_millis(16);
        if elapsed < target_frame_time {
            thread::sleep(target_frame_time - elapsed);
        }
    }

    // Cleanup & restore terminal
    execute!(stdout, Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    println!("Cavatrix finished.");

    Ok(())
}
