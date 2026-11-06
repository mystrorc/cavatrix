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
    pub show_bottom_wave: bool,
}

impl MatrixEngine {
    pub fn new(width: usize, height: usize) -> Self {
        let mut engine = Self {
            width,
            height,
            columns: vec![Vec::new(); width],
            spawn_cooldowns: vec![0.0; width],
            speed_multiplier: 1.0,
            show_bottom_wave: true,
        };

        // Seed initial streams with varying heights
        for col in 0..width {
            if fastrand::f32() < 0.75 {
                let initial_y = fastrand::f32() * (height as f32);
                let length = fastrand::usize(10..=(height * 3 / 4).max(12));
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
    }

    pub fn update(&mut self, dt: f32, get_energy: impl Fn(usize) -> f32) {
        for col in 0..self.width {
            let energy = get_energy(col);

            // Update existing streams in this column
            for stream in &mut self.columns[col] {
                stream.update(dt, energy, self.speed_multiplier);
            }

            // Remove streams that have completely fallen off the bottom
            let h = self.height;
            self.columns[col].retain(|stream| stream.is_alive(h));

            // Handle spawn cooldown
            self.spawn_cooldowns[col] -= dt;

            // Spawning probability increases with audio energy
            let can_spawn = self.columns[col].len() < 2
                && (self.columns[col].is_empty()
                    || self.columns[col].last().unwrap().head_y > 10.0);

            if can_spawn && self.spawn_cooldowns[col] <= 0.0 {
                let length = fastrand::usize(8..=(self.height * 2 / 3).max(12));
                let base_speed = fastrand::f32() * 14.0 + 7.0;
                let start_y = -(fastrand::f32() * 4.0);

                self.columns[col].push(DropStream::new(start_y, length, base_speed));

                // Audio beats reduce cooldown so more rain pours down during drops
                let cooldown_base = fastrand::f32() * 2.0 + 0.5;
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

        // Draw streams from oldest to newest
        for col in 0..self.width {
            let energy = get_energy(col);

            for stream in &self.columns[col] {
                let head_int = stream.head_y.floor() as i32;

                for (idx, &ch) in stream.chars.iter().enumerate() {
                    let y = head_int - idx as i32;
                    if y >= 0 && (y as usize) < self.height {
                        let y_u = y as usize;
                        let is_head = idx == 0;
                        let tail_factor = 1.0 - (idx as f32 / stream.length as f32);

                        let color = color::get_character_color(
                            palette,
                            col,
                            self.width,
                            energy,
                            tail_factor,
                            is_head,
                        );

                        // If cell is already drawn, pick brighter color
                        let existing = grid[y_u][col].1;
                        let existing_lum = existing.r as u32 + existing.g as u32 + existing.b as u32;
                        let new_lum = color.r as u32 + color.g as u32 + color.b as u32;

                        if new_lum >= existing_lum {
                            grid[y_u][col] = (ch, color);
                        }
                    }
                }
            }
        }

        // Optional bottom CAVA equalizer baseline indicator
        if self.show_bottom_wave && self.height > 2 {
            let bottom_row = self.height - 1;
            const BLOCKS: &[char] = &[' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

            for col in 0..self.width {
                let energy = get_energy(col);
                let block_idx = ((energy * 8.0).round() as usize).min(BLOCKS.len() - 1);
                let block_char = BLOCKS[block_idx];

                if block_char != ' ' {
                    let wave_color = color::get_character_color(
                        palette,
                        col,
                        self.width,
                        energy,
                        1.0,
                        false,
                    );
                    grid[bottom_row][col] = (block_char, wave_color);
                }
            }
        }

        grid
    }
}
