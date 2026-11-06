#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    MatrixSupernova,
    CyberpunkSynthwave,
    RainbowWave,
    IceAndFire,
    AmaneSunset,
}

impl PaletteMode {
    pub fn next(&self) -> Self {
        match self {
            Self::MatrixSupernova => Self::CyberpunkSynthwave,
            Self::CyberpunkSynthwave => Self::RainbowWave,
            Self::RainbowWave => Self::IceAndFire,
            Self::IceAndFire => Self::AmaneSunset,
            Self::AmaneSunset => Self::MatrixSupernova,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::MatrixSupernova => "Matrix Supernova",
            Self::CyberpunkSynthwave => "Cyberpunk Synthwave",
            Self::RainbowWave => "Rainbow Spectrum",
            Self::IceAndFire => "Ice & Fire",
            Self::AmaneSunset => "Amane Sunset",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorRgb {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: (a.r as f32 + (b.r as f32 - a.r as f32) * t).round() as u8,
            g: (a.g as f32 + (b.g as f32 - a.g as f32) * t).round() as u8,
            b: (a.b as f32 + (b.b as f32 - a.b as f32) * t).round() as u8,
        }
    }

    pub fn scale(&self, factor: f32) -> Self {
        let factor = factor.max(0.0);
        Self {
            r: ((self.r as f32 * factor).min(255.0)).round() as u8,
            g: ((self.g as f32 * factor).min(255.0)).round() as u8,
            b: ((self.b as f32 * factor).min(255.0)).round() as u8,
        }
    }

    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = (h % 360.0 + 360.0) % 360.0;
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: ((r1 + m) * 255.0).clamp(0.0, 255.0).round() as u8,
            g: ((g1 + m) * 255.0).clamp(0.0, 255.0).round() as u8,
            b: ((b1 + m) * 255.0).clamp(0.0, 255.0).round() as u8,
        }
    }
}

pub fn get_character_color(
    mode: PaletteMode,
    column: usize,
    total_cols: usize,
    audio_energy: f32, // 0.0 .. 1.0 for this column
    tail_factor: f32,  // 1.0 at head -> 0.0 at end of tail
    is_head: bool,
) -> ColorRgb {
    if is_head {
        if audio_energy > 0.85 {
            // Intense energy: pure blazing white with slight tint
            return ColorRgb::WHITE;
        } else {
            // Moderately bright head
            let head_tint = match mode {
                PaletteMode::MatrixSupernova => ColorRgb::new(200, 255, 210),
                PaletteMode::CyberpunkSynthwave => ColorRgb::new(255, 210, 255),
                PaletteMode::RainbowWave => ColorRgb::new(255, 255, 255),
                PaletteMode::IceAndFire => ColorRgb::new(220, 245, 255),
                PaletteMode::AmaneSunset => ColorRgb::new(255, 235, 220),
            };
            return ColorRgb::lerp(head_tint, ColorRgb::WHITE, 0.7);
        }
    }

    let base_color = match mode {
        PaletteMode::MatrixSupernova => {
            // Matrix phosphor green -> electric lime -> cyan -> hot orange -> supernova pink
            let green_dim = ColorRgb::new(0, 110, 30);
            let green_bright = ColorRgb::new(20, 255, 80);
            let electric_lime = ColorRgb::new(140, 255, 20);
            let neon_cyan = ColorRgb::new(0, 230, 255);
            let hot_orange = ColorRgb::new(255, 110, 0);
            let hot_pink = ColorRgb::new(255, 20, 110);

            if audio_energy < 0.25 {
                let t = audio_energy / 0.25;
                ColorRgb::lerp(green_dim, green_bright, t)
            } else if audio_energy < 0.50 {
                let t = (audio_energy - 0.25) / 0.25;
                ColorRgb::lerp(green_bright, electric_lime, t)
            } else if audio_energy < 0.75 {
                let t = (audio_energy - 0.50) / 0.25;
                ColorRgb::lerp(electric_lime, neon_cyan, t)
            } else if audio_energy < 0.90 {
                let t = (audio_energy - 0.75) / 0.15;
                ColorRgb::lerp(neon_cyan, hot_orange, t)
            } else {
                let t = (audio_energy - 0.90) / 0.10;
                ColorRgb::lerp(hot_orange, hot_pink, t)
            }
        }

        PaletteMode::CyberpunkSynthwave => {
            // Deep purple -> Electric violet -> Neon cyan -> Hot pink
            let dark_purple = ColorRgb::new(60, 10, 100);
            let electric_violet = ColorRgb::new(160, 20, 255);
            let neon_cyan = ColorRgb::new(0, 240, 255);
            let hot_pink = ColorRgb::new(255, 0, 140);

            if audio_energy < 0.35 {
                let t = audio_energy / 0.35;
                ColorRgb::lerp(dark_purple, electric_violet, t)
            } else if audio_energy < 0.70 {
                let t = (audio_energy - 0.35) / 0.35;
                ColorRgb::lerp(electric_violet, neon_cyan, t)
            } else {
                let t = (audio_energy - 0.70) / 0.30;
                ColorRgb::lerp(neon_cyan, hot_pink, t)
            }
        }

        PaletteMode::RainbowWave => {
            // Frequency / position drives hue; amplitude drives saturation & brightness
            let col_ratio = column as f32 / total_cols.max(1) as f32;
            let hue = (col_ratio * 360.0 + audio_energy * 120.0) % 360.0;
            let sat = (0.7 + 0.3 * audio_energy).clamp(0.0, 1.0);
            let val = (0.5 + 0.5 * audio_energy).clamp(0.0, 1.0);
            ColorRgb::from_hsv(hue, sat, val)
        }

        PaletteMode::IceAndFire => {
            // Glacial arctic blue/cyan -> Blazing amber flame & lava
            let deep_blue = ColorRgb::new(15, 60, 180);
            let ice_cyan = ColorRgb::new(50, 200, 255);
            let flame_gold = ColorRgb::new(255, 180, 20);
            let lava_red = ColorRgb::new(255, 40, 10);

            if audio_energy < 0.40 {
                let t = audio_energy / 0.40;
                ColorRgb::lerp(deep_blue, ice_cyan, t)
            } else if audio_energy < 0.75 {
                let t = (audio_energy - 0.40) / 0.35;
                ColorRgb::lerp(ice_cyan, flame_gold, t)
            } else {
                let t = (audio_energy - 0.75) / 0.25;
                ColorRgb::lerp(flame_gold, lava_red, t)
            }
        }

        PaletteMode::AmaneSunset => {
            // Twilight velvet -> Amane coral #f56d66 -> Sunrise gold
            let velvet = ColorRgb::new(50, 20, 80);
            let amane_coral = ColorRgb::new(245, 109, 102);
            let warm_gold = ColorRgb::new(255, 215, 100);

            if audio_energy < 0.50 {
                let t = audio_energy / 0.50;
                ColorRgb::lerp(velvet, amane_coral, t)
            } else {
                let t = (audio_energy - 0.50) / 0.50;
                ColorRgb::lerp(amane_coral, warm_gold, t)
            }
        }
    };

    // Apply tail decay: characters get dimmer down the tail, with audio energy preventing dropoff
    let decay_curve = tail_factor.powf(1.6);
    let min_visibility = 0.15 + 0.25 * audio_energy;
    let brightness = (decay_curve * (1.0 - min_visibility) + min_visibility).clamp(0.0, 1.0);

    base_color.scale(brightness)
}
