use crate::color::{self, ColorRgb, PaletteMode};

// Authentic half-width katakana, numeric digits, and matrix symbols
const GLYPHS: &[char] = &[
    'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｰ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ',
    'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ',
    'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ',
    'ﾜ', 'ﾝ', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    'X', 'Y', 'Z', '@', '#', '$', '%', '&', '*', '+', '=', '<', '>', '~', ':', ';',
];

fn random_glyph() -> char {
    let idx = fastrand::usize(..GLYPHS.len());
    GLYPHS[idx]
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BarStyle {
    Solid,  // '█'
    Shaded, // '▒'
    Dense,  // '▓'
    Pillar, // '┃'
}

impl BarStyle {
    pub fn next(&self) -> Self {
        match self {
            Self::Solid => Self::Shaded,
            Self::Shaded => Self::Dense,
            Self::Dense => Self::Pillar,
            Self::Pillar => Self::Solid,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Solid => "Solid (█)",
            Self::Shaded => "Shaded (▒)",
            Self::Dense => "Dense (▓)",
            Self::Pillar => "Pillar (┃)",
        }
    }

    pub fn fill_char(&self) -> char {
        match self {
            Self::Solid => '█',
            Self::Shaded => '▒',
            Self::Dense => '▓',
            Self::Pillar => '┃',
        }
    }
}

#[derive(Clone)]
struct DropStream {
    head_y: f32,
    base_speed: f32, // rows per second
    length: usize,
    chars: Vec<char>,
    mutation_timers: Vec<f32>,
}

impl DropStream {
    fn new(start_y: f32, length: usize, base_speed: f32) -> Self {
        let chars: Vec<char> = (0..length).map(|_| random_glyph()).collect();
        let mutation_timers: Vec<f32> = (0..length)
            .map(|_| fastrand::f32() * 0.4 + 0.1)
            .collect();

        Self {
            head_y: start_y,
            base_speed,
            length,
            chars,
            mutation_timers,
        }
    }

    fn update(&mut self, dt: f32, audio_energy: f32, speed_multiplier: f32) {
        // Audio energy dynamically accelerates the falling stream
        let speed_boost = 1.0 + (audio_energy * 1.5);
        self.head_y += self.base_speed * speed_boost * speed_multiplier * dt;

        // Mutate random characters along the tail for the Matrix glitch effect
        for (i, timer) in self.mutation_timers.iter_mut().enumerate() {
            *timer -= dt;
            if *timer <= 0.0 {
                if fastrand::f32() < (0.2 + 0.5 * audio_energy) {
                    self.chars[i] = random_glyph();
                }
                *timer = fastrand::f32() * 0.5 + 0.1;
            }
        }
    }

    fn is_alive(&self, height: usize) -> bool {
        (self.head_y - self.length as f32) < height as f32
    }
}

pub struct MatrixEngine {
    pub width: usize,
    pub height: usize,
    columns: Vec<Vec<DropStream>>,
    spawn_cooldowns: Vec<f32>,
    pub speed_multiplier: f32,
    pub show_cava_bars: bool,
    pub bar_style: BarStyle,
    pub max_bar_ratio: f32, // fraction of screen height bars can reach (e.g. 0.60 = 60%)
    peaks: Vec<f32>,
}

impl MatrixEngine {
    pub fn new(width: usize, height: usize) -> Self {
        let mut engine = Self {
            width,
            height,
            columns: vec![Vec::new(); width],
            spawn_cooldowns: vec![0.0; width],
            speed_multiplier: 1.0,
            show_cava_bars: true,
            bar_style: BarStyle::Solid,
            max_bar_ratio: 0.65, // Extends up to 65% of screen (well over half screen)
            peaks: vec![0.0; width],
        };

        // Seed initial streams with varying heights
        for col in 0..width {
            if fastrand::f32() < 0.75 {
                let initial_y = fastrand::f32() * (height as f32);
                let length = fastrand::usize(12..=(height * 3 / 4).max(14));
                let speed = fastrand::f32() * 12.0 + 8.0;
                engine.columns[col].push(DropStream::new(initial_y, length, speed));
            }
            engine.spawn_cooldowns[col] = fastrand::f32() * 1.5;
        }

        engine
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        self.columns.resize(new_width, Vec::new());
        self.spawn_cooldowns.resize(new_width, 0.0);
        self.peaks.resize(new_width, 0.0);
    }

    pub fn update(&mut self, dt: f32, get_energy: impl Fn(usize) -> f32) {
        let max_bar_h = (self.height as f32 * self.max_bar_ratio).max(4.0);

        for col in 0..self.width {
            let energy = get_energy(col);
            let current_bar_h = energy * max_bar_h;

            // Update floating peak caps
            if current_bar_h > self.peaks[col] {
                self.peaks[col] = current_bar_h;
            } else {
                self.peaks[col] = (self.peaks[col] - (dt * 14.0)).max(0.0);
            }

            // Update existing rain streams in this column
            for stream in &mut self.columns[col] {
                stream.update(dt, energy, self.speed_multiplier);
            }

            // Remove streams that have completely fallen off the bottom
            let h = self.height;
            self.columns[col].retain(|stream| stream.is_alive(h));

            // Handle spawn cooldown
            self.spawn_cooldowns[col] -= dt;

            // Spawning probability increases with audio energy
            let can_spawn = self.columns[col].len() < 3
                && (self.columns[col].is_empty()
                    || self.columns[col].last().unwrap().head_y > 8.0);

            if can_spawn && self.spawn_cooldowns[col] <= 0.0 {
                let length = fastrand::usize(10..=(self.height * 4 / 5).max(14));
                let base_speed = fastrand::f32() * 14.0 + 7.0;
                let start_y = -(fastrand::f32() * 5.0);

                self.columns[col].push(DropStream::new(start_y, length, base_speed));

                let cooldown_base = fastrand::f32() * 1.8 + 0.4;
                let cooldown_reduction = (1.0 - energy * 0.7).clamp(0.2, 1.0);
                self.spawn_cooldowns[col] = cooldown_base * cooldown_reduction;
            }
        }
    }

    pub fn render_frame(
        &self,
        palette: PaletteMode,
        get_energy: impl Fn(usize) -> f32,
    ) -> Vec<Vec<(char, ColorRgb)>> {
        let mut grid = vec![vec![(' ', ColorRgb::BLACK); self.width]; self.height];

        let max_bar_h = (self.height as f32 * self.max_bar_ratio).max(4.0);
        const BLOCKS: &[char] = &[' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇'];

        // 1. Render Multi-Row CAVA Bars (Rising up to half/two-thirds screen)
        if self.show_cava_bars && self.height > 4 {
            let fill_glyph = self.bar_style.fill_char();

            for col in 0..self.width {
                let energy = get_energy(col);
                if energy <= 0.01 {
                    continue;
                }

                let bar_h = (energy * max_bar_h).min((self.height - 1) as f32);
                let full_rows = bar_h.floor() as usize;
                let frac = bar_h - bar_h.floor();

                // Draw solid/shaded bar rows from bottom upwards
                for dy in 0..full_rows {
                    if dy >= self.height {
                        break;
                    }
                    let y = self.height - 1 - dy;
                    let height_ratio = dy as f32 / max_bar_h;

                    // Gradient: brighter/hotter towards the top of the bar
                    let bar_color = color::get_character_color(
                        palette,
                        col,
                        self.width,
                        (energy * 0.7 + height_ratio * 0.3).clamp(0.0, 1.0),
                        0.7 + 0.3 * height_ratio,
                        false,
                    );

                    grid[y][col] = (fill_glyph, bar_color);
                }

                // Draw smooth fractional block at the top peak of the bar
                let frac_idx = ((frac * 8.0).floor() as usize).min(BLOCKS.len() - 1);
                let top_block = BLOCKS[frac_idx];
                if top_block != ' ' && full_rows < self.height {
                    let y = self.height - 1 - full_rows;
                    let peak_color = color::get_character_color(
                        palette,
                        col,
                        self.width,
                        energy,
                        1.0,
                        false,
                    );
                    grid[y][col] = (top_block, peak_color);
                }

                // Draw floating peak cap
                let peak_h = self.peaks[col].min((self.height - 1) as f32);
                let peak_y = (self.height - 1).saturating_sub(peak_h.round() as usize);
                if peak_y < self.height && grid[peak_y][col].0 == ' ' {
                    let cap_color = ColorRgb::lerp(
                        color::get_character_color(palette, col, self.width, energy, 1.0, true),
                        ColorRgb::WHITE,
                        0.6,
                    );
                    grid[peak_y][col] = ('▔', cap_color);
                }
            }
        }

        // 2. Render Matrix Digital Rain (Falling through all the way to bottom)
        for col in 0..self.width {
            let energy = get_energy(col);
            let bar_h = if self.show_cava_bars {
                (energy * max_bar_h) as usize
            } else {
                0
            };

            for stream in &self.columns[col] {
                let head_int = stream.head_y.floor() as i32;

                for (idx, &ch) in stream.chars.iter().enumerate() {
                    let y = head_int - idx as i32;
                    if y >= 0 && (y as usize) < self.height {
                        let y_u = y as usize;
                        let is_head = idx == 0;
                        let tail_factor = 1.0 - (idx as f32 / stream.length as f32);

                        let mut color = color::get_character_color(
                            palette,
                            col,
                            self.width,
                            energy,
                            tail_factor,
                            is_head,
                        );

                        // If glyph is inside an active CAVA bar, boost its contrast & luminosity
                        let rows_from_bottom = self.height - 1 - y_u;
                        let inside_bar = rows_from_bottom <= bar_h;

                        if inside_bar {
                            if is_head {
                                color = ColorRgb::WHITE;
                            } else {
                                // Supercharge glyph color inside the bar so it pops dramatically
                                color = ColorRgb::lerp(color, ColorRgb::WHITE, 0.45);
                            }
                        }

                        // Matrix rain character always takes precedence over bar fill
                        grid[y_u][col] = (ch, color);
                    }
                }
            }
        }

        grid
    }
}
