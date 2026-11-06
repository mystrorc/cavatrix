use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

pub struct AudioProcessor {
    spectrum: Arc<Mutex<Vec<f32>>>,
    smoothed: Vec<f32>,
    pub gain: f32,
    pub stereo: bool,
    _cava_child: Option<Child>,
    start_time: Instant,
    using_synthetic: Arc<Mutex<bool>>,
}

impl AudioProcessor {
    pub fn new(bar_count: usize) -> Self {
        let spectrum = Arc::new(Mutex::new(vec![0.0f32; bar_count]));
        let using_synthetic = Arc::new(Mutex::new(false));

        let child = Self::start_cava_listener(bar_count, spectrum.clone(), using_synthetic.clone());

        Self {
            spectrum,
            smoothed: vec![0.0f32; bar_count],
            gain: 1.0,
            stereo: true,
            _cava_child: child,
            start_time: Instant::now(),
            using_synthetic,
        }
    }

    fn start_cava_listener(
        bar_count: usize,
        spectrum: Arc<Mutex<Vec<f32>>>,
        using_synthetic: Arc<Mutex<bool>>,
    ) -> Option<Child> {
        let cava_config = format!(
            r#"[general]
bars = {bars}
framerate = 60
autosens = 1

[input]
method = pulse
source = auto

[output]
method = raw
raw_target = /dev/stdout
data_format = binary
bit_format = 8bit
"#,
            bars = bar_count
        );

        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("cavatrix_cava.conf");
        if std::fs::write(&config_path, cava_config).is_err() {
            *using_synthetic.lock().unwrap() = true;
            return None;
        }

        let child = Command::new("cava")
            .arg("-p")
            .arg(&config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        match child {
            Ok(mut process) => {
                let mut stdout = process.stdout.take().unwrap();
                let spec_clone = spectrum.clone();
                let syn_clone = using_synthetic.clone();

                thread::spawn(move || {
                    let mut buffer = vec![0u8; bar_count];
                    let mut all_zeros_count = 0;

                    loop {
                        match stdout.read_exact(&mut buffer) {
                            Ok(()) => {
                                let is_zero = buffer.iter().all(|&b| b == 0);
                                if is_zero {
                                    all_zeros_count += 1;
                                    if all_zeros_count > 60 {
                                        *syn_clone.lock().unwrap() = true;
                                    }
                                } else {
                                    all_zeros_count = 0;
                                    *syn_clone.lock().unwrap() = false;
                                }

                                if let Ok(mut spec) = spec_clone.lock() {
                                    for (i, &byte_val) in buffer.iter().enumerate() {
                                        if i < spec.len() {
                                            spec[i] = (byte_val as f32) / 255.0;
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                *syn_clone.lock().unwrap() = true;
                                break;
                            }
                        }
                    }
                });

                Some(process)
            }
            Err(_) => {
                *using_synthetic.lock().unwrap() = true;
                None
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        let is_synth = *self.using_synthetic.lock().unwrap();

        if is_synth {
            // Generate dynamic rhythmic harmonics so visualizer pulses naturally when silent
            let t = self.start_time.elapsed().as_secs_f32();
            let count = self.smoothed.len();

            for i in 0..count {
                let frac = i as f32 / count as f32;
                let wave1 = (t * 2.5 + frac * 6.28).sin().max(0.0);
                let wave2 = (t * 4.0 - frac * 12.0).cos().max(0.0) * 0.5;
                let beat = (t * 2.0).sin().powi(4);
                let val = (wave1 + wave2) * (0.3 + 0.7 * beat);
                self.smoothed[i] = (val * self.gain).clamp(0.0, 1.0);
            }
            return;
        }

        if let Ok(spec) = self.spectrum.lock() {
            let decay = (1.0 - (dt * 12.0)).clamp(0.1, 0.95);
            let attack = (dt * 28.0).clamp(0.05, 1.0);

            for (i, &target) in spec.iter().enumerate() {
                if i < self.smoothed.len() {
                    let target_gained = (target * self.gain).clamp(0.0, 1.0);
                    if target_gained > self.smoothed[i] {
                        self.smoothed[i] += (target_gained - self.smoothed[i]) * attack;
                    } else {
                        self.smoothed[i] += (target_gained - self.smoothed[i]) * (1.0 - decay);
                    }
                }
            }
        }
    }

    pub fn get_column_energy(&self, col: usize, total_cols: usize) -> f32 {
        if total_cols == 0 || self.smoothed.is_empty() {
            return 0.0;
        }

        let bar_count = self.smoothed.len();

        let bar_index = if self.stereo {
            // Stereo mirror: Center has bass (low frequencies), edges have treble
            let center = total_cols as f32 / 2.0;
            let dist_from_center = (col as f32 - center).abs() / center; // 0.0 at center, 1.0 at edges
            (dist_from_center * (bar_count - 1) as f32).round() as usize
        } else {
            // Left to right: Bass on left -> Treble on right
            let ratio = col as f32 / total_cols as f32;
            (ratio * (bar_count - 1) as f32).round() as usize
        };

        let clamped_idx = bar_index.min(bar_count - 1);
        self.smoothed[clamped_idx]
    }

    #[allow(dead_code)]
    pub fn get_overall_energy(&self) -> f32 {
        if self.smoothed.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.smoothed.iter().sum();
        sum / self.smoothed.len() as f32
    }

    #[allow(dead_code)]
    pub fn get_bass_energy(&self) -> f32 {
        if self.smoothed.is_empty() {
            return 0.0;
        }
        let bass_count = (self.smoothed.len() / 5).max(1);
        let sum: f32 = self.smoothed[..bass_count].iter().sum();
        sum / bass_count as f32
    }

    #[allow(dead_code)]
    pub fn is_synthetic(&self) -> bool {
        *self.using_synthetic.lock().unwrap()
    }
}

impl Drop for AudioProcessor {
    fn drop(&mut self) {
        if let Some(mut child) = self._cava_child.take() {
            let _ = child.kill();
        }
    }
}
