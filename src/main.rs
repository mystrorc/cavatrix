mod audio;
mod color;
mod matrix;

use std::io::{self, BufWriter, Write};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, Show};
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

fn print_help() {
    println!(
        r#"cavatrix {version}
Audio-reactive terminal visualizer combining CAVA frequency spectrum with cmatrix digital rain.

USAGE:
    cavatrix [OPTIONS]

OPTIONS:
    -f, --fps <FPS>       Target framerate in frames per second [default: 30] (e.g. 30 or 60)
    -b, --bars <BARS>     Number of audio frequency bars [default: 64]
    -m, --mono            Start in mono audio mode (Bass on left -> Treble on right) [default: stereo]
    -s, --speed <SPEED>   Initial rain falling speed multiplier [default: 1.0]
    -h, --help            Print help information
    -v, --version         Print version information

INTERACTIVE CONTROLS:
    q, Esc       Exit cavatrix
    c            Cycle color palette (Supernova, Cyberpunk, Fire, Synthwave, etc.)
    v            Cycle CAVA bar style (Solid █, Shaded ▒, Dense ▓, Pillar ┃)
    b            Toggle CAVA equalizer bars ON/OFF
    Up / Down    Adjust maximum bar height (20% - 95% of screen height)
    m            Toggle stereo (bass in center) / mono audio mapping
    + / = / -    Adjust audio gain / sensitivity
    [ / ]        Adjust rain falling speed
    Space        Pause / resume visualizer
    h, ?         Toggle in-visualizer help overlay"#,
        version = env!("CARGO_PKG_VERSION")
    );
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut fps: u32 = 30;
    let mut bar_count: usize = 64;
    let mut rain_speed: f32 = 1.0;
    let mut start_mono: bool = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-v" | "--version" => {
                println!("cavatrix {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-f" | "--fps" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Ok(val) = args[i].parse::<u32>() {
                        fps = val.clamp(10, 120);
                    } else {
                        eprintln!(
                            "Error: Invalid FPS value '{}'. Must be an integer (e.g. 30, 60).\n",
                            args[i]
                        );
                        print_help();
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: Missing value for --fps option.\n");
                    print_help();
                    std::process::exit(1);
                }
            }
            "-b" | "--bars" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Ok(val) = args[i].parse::<usize>() {
                        bar_count = val.clamp(16, 256);
                    } else {
                        eprintln!("Error: Invalid bars value '{}'. Must be an integer.\n", args[i]);
                        print_help();
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: Missing value for --bars option.\n");
                    print_help();
                    std::process::exit(1);
                }
            }
            "-s" | "--speed" => {
                if i + 1 < args.len() {
                    i += 1;
                    if let Ok(val) = args[i].parse::<f32>() {
                        rain_speed = val.clamp(0.1, 5.0);
                    } else {
                        eprintln!("Error: Invalid speed value '{}'.\n", args[i]);
                        print_help();
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: Missing value for --speed option.\n");
                    print_help();
                    std::process::exit(1);
                }
            }
            "-m" | "--mono" => {
                start_mono = true;
            }
            other => {
                eprintln!("Error: Unknown option '{}'.\n", other);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Install panic hook to ensure terminal state is always restored
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        default_panic(info);
    }));

    enable_raw_mode()?;
    // Generous output buffer (128KB) to batch terminal writes into large chunks
    let mut stdout = BufWriter::with_capacity(131072, io::stdout());
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        crossterm::terminal::SetTitle("cavatrix")
    )?;

    let (mut cols, mut rows) = crossterm::terminal::size()?;
    let mut total_cells = (cols as usize) * (rows as usize);

    let mut engine = MatrixEngine::new(cols as usize, rows as usize);
    engine.speed_multiplier = rain_speed;

    let mut audio = AudioProcessor::new(bar_count, fps);
    if start_mono {
        audio.stereo = false;
    }

    let mut palette = PaletteMode::MatrixSupernova;
    let mut paused = false;
    let mut show_help = false;
    let mut osd: Option<OsdMessage> = Some(OsdMessage {
        text: format!("CAVATRIX: Audio-Reactive Matrix Rain @ {} FPS (Press 'h' for help)", fps),
        expiry: Instant::now() + Duration::from_secs(4),
    });

    let mut last_frame = Instant::now();

    // Reusable double-buffer grids to eliminate per-frame heap allocations entirely
    let mut prev_grid: Vec<(char, ColorRgb)> = vec![(' ', ColorRgb::BLACK); total_cells];
    let mut curr_grid: Vec<(char, ColorRgb)> = vec![(' ', ColorRgb::BLACK); total_cells];

    let mut ch_buf = [0u8; 4];
    let target_frame_time = Duration::from_micros(1_000_000 / fps as u64);

    'main_loop: loop {
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f32().min(0.1);
        last_frame = now;
        let frame_start = Instant::now();

        // Handle user keyboard input non-blocking
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) => {
                    // Check for Ctrl+C or 'q' / Esc to exit
                    if key_event.modifiers.contains(KeyModifiers::CONTROL)
                        && key_event.code == KeyCode::Char('c')
                    {
                        break 'main_loop;
                    }

                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break 'main_loop;
                        }
                        KeyCode::Char('c') => {
                            palette = palette.next();
                            osd = Some(OsdMessage {
                                text: format!("Palette: {}", palette.name()),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('v') => {
                            engine.bar_style = engine.bar_style.next();
                            osd = Some(OsdMessage {
                                text: format!("Bar Style: {}", engine.bar_style.name()),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('b') => {
                            engine.show_cava_bars = !engine.show_cava_bars;
                            osd = Some(OsdMessage {
                                text: format!(
                                    "CAVA Equalizer Bars: {}",
                                    if engine.show_cava_bars { "ON" } else { "OFF" }
                                ),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Up => {
                            engine.max_bar_ratio = (engine.max_bar_ratio + 0.05).min(0.95);
                            osd = Some(OsdMessage {
                                text: format!(
                                    "Max Bar Height: {:.0}% screen",
                                    engine.max_bar_ratio * 100.0
                                ),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Down => {
                            engine.max_bar_ratio = (engine.max_bar_ratio - 0.05).max(0.20);
                            osd = Some(OsdMessage {
                                text: format!(
                                    "Max Bar Height: {:.0}% screen",
                                    engine.max_bar_ratio * 100.0
                                ),
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
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            audio.gain = (audio.gain * 1.25).min(5.0);
                            osd = Some(OsdMessage {
                                text: format!("Audio Gain: {:.1}x", audio.gain),
                                expiry: Instant::now() + Duration::from_secs(2),
                            });
                        }
                        KeyCode::Char('-') => {
                            audio.gain = (audio.gain / 1.25).max(0.2);
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
                                text: if paused {
                                    "Paused".to_string()
                                } else {
                                    "Resumed".to_string()
                                },
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
                    total_cells = (cols as usize) * (rows as usize);
                    engine.resize(cols as usize, rows as usize);
                    prev_grid = vec![(' ', ColorRgb::BLACK); total_cells];
                    curr_grid = vec![(' ', ColorRgb::BLACK); total_cells];
                    let _ = execute!(
                        stdout,
                        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
                    );
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

        // Render matrix directly into flat slice with zero allocations
        let width = engine.width;
        engine.render_into(
            palette,
            |col| audio.get_column_energy(col, width),
            &mut curr_grid,
        );

        // Overlay OSD notification if active
        if let Some(ref msg) = osd {
            if Instant::now() < msg.expiry {
                let banner = format!(" [ {} ] ", msg.text);
                let banner_len = banner.chars().count();
                if banner_len < cols as usize && rows > 3 {
                    let start_col = (cols as usize - banner_len) / 2;
                    let target_row = 1;
                    for (i, ch) in banner.chars().enumerate() {
                        let idx = target_row * (cols as usize) + (start_col + i);
                        if idx < total_cells {
                            curr_grid[idx] = (ch, ColorRgb::new(255, 255, 255));
                        }
                    }
                }
            }
        }

        // Overlay Help Legend if active
        if show_help && rows > 11 && cols > 48 {
            let help_lines = [
                "┌─────────────────── CAVATRIX HELP ───────────────────┐",
                "│  q / Esc   - Exit cavatrix                          │",
                "│  c         - Cycle color palette                    │",
                "│  v         - Cycle bar style (Solid/Shaded/Pillar)  │",
                "│  b         - Toggle CAVA equalizer bars ON/OFF      │",
                "│  Up / Down - Adjust max bar height (up to 90%)      │",
                "│  m         - Toggle stereo / mono audio mapping     │",
                "│  + / -     - Adjust audio gain / sensitivity        │",
                "│  [ / ]     - Adjust rain falling speed              │",
                "│  Space     - Pause / Resume visualizer              │",
                "│  h / ?     - Toggle this help dialog                │",
                "└─────────────────────────────────────────────────────┘",
            ];
            let start_row = (rows as usize - help_lines.len()) / 2;
            let start_col = (cols as usize - 55) / 2;
            for (r_idx, line) in help_lines.iter().enumerate() {
                for (c_idx, ch) in line.chars().enumerate() {
                    let r = start_row + r_idx;
                    let c = start_col + c_idx;
                    if r < rows as usize && c < cols as usize {
                        let idx = r * (cols as usize) + c;
                        if idx < total_cells {
                            curr_grid[idx] = (ch, ColorRgb::new(200, 250, 255));
                        }
                    }
                }
            }
        }

        // High-Performance Sequential Terminal Renderer:
        // Only emit MoveTo when the cursor actually has to jump (skipping unchanged cells).
        // For continuous runs of updated characters, the terminal cursor advances automatically!
        let mut cursor_r: usize = usize::MAX;
        let mut cursor_c: usize = usize::MAX;
        let mut current_fg: Option<ColorRgb> = None;

        let w = cols as usize;
        let h = rows as usize;

        for r in 0..h {
            let row_offset = r * w;
            for c in 0..w {
                let idx = row_offset + c;
                let (ch, col_rgb) = curr_grid[idx];
                let (prev_ch, prev_col) = prev_grid[idx];

                if ch != prev_ch || col_rgb != prev_col {
                    // Only jump the cursor if it isn't already sitting at (r, c)
                    if cursor_r != r || cursor_c != c {
                        let _ = write!(stdout, "\x1b[{};{}H", r + 1, c + 1);
                        cursor_r = r;
                        cursor_c = c;
                    }

                    // Only emit color escape sequence if color changed
                    if current_fg != Some(col_rgb) {
                        let _ = write!(
                            stdout,
                            "\x1b[38;2;{};{};{}m",
                            col_rgb.r, col_rgb.g, col_rgb.b
                        );
                        current_fg = Some(col_rgb);
                    }

                    let ch_str = ch.encode_utf8(&mut ch_buf);
                    let _ = stdout.write_all(ch_str.as_bytes());
                    cursor_c += 1; // Terminal cursor naturally advances by 1
                }
            }
        }

        let _ = stdout.flush();
        prev_grid.copy_from_slice(&curr_grid);

        let elapsed = frame_start.elapsed();
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
