// --- Core Game Logic (backend-agnostic) ---
use bevy::prelude::{Color, ColorToPacked, Resource};
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;

/// Converts a 2D coordinate to a 1D vector index.
pub fn get_index(width: usize, row: usize, column: usize) -> usize {
    row * width + column
}

/// Counts the number of live neighbors and collects their colors (toroidal wrapping).
fn get_live_neighbors_info(
    cells: &[Option<Color>],
    width: usize,
    height: usize,
    row: usize,
    column: usize,
) -> (u8, Vec<Color>) {
    let mut count = 0;
    let mut colors = Vec::with_capacity(8);
    // Iterate over a 3x3 grid centered on the cell
    for delta_row in [height - 1, 0, 1].iter() {
        for delta_col in [width - 1, 0, 1].iter() {
            // Skip the cell itself
            if *delta_row == 0 && *delta_col == 0 {
                continue;
            }
            let neighbor_row = (row + delta_row) % height;
            let neighbor_col = (column + delta_col) % width;
            let idx = get_index(width, neighbor_row, neighbor_col);
            if let Some(color) = cells[idx] {
                count += 1;
                colors.push(color);
            }
        }
    }
    (count, colors)
}

/// Represents the game world.
#[derive(Resource)]
pub struct Game {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub cells: Vec<Option<Color>>, // Changed from Vec<bool> to Vec<Option<Color>>
    next_cells: Vec<Option<Color>>,
    genesis_interval: u32,
    genesis_cluster_size: u32,
    genesis_density: f32,
    tick_counter: u32,
}

impl Game {
    /// Creates a new Game instance.
    pub fn new(
        width: usize,
        height: usize,
        cell_size: f32,
        initial_density: f32,
        initial_color: Color,
        genesis_interval: u32,
        genesis_cluster_size: u32,
        genesis_density: f32,
    ) -> Game {
        let size = width * height;
        let mut cells = vec![None; size];
        let mut rng = rand::rng();
        for cell in cells.iter_mut() {
            if rng.random_bool(initial_density as f64) {
                *cell = Some(initial_color);
            }
        }

        Game {
            width,
            height,
            cell_size,
            cells,
            next_cells: vec![None; size],
            genesis_interval,
            genesis_cluster_size,
            genesis_density,
            tick_counter: 0,
        }
    }

    /// Creates a new random cluster of life with a new random color.
    fn random_genesis(&mut self) {
        let mut rng = rand::rng();
        let cluster_size = self.genesis_cluster_size as usize;
        if self.width <= cluster_size || self.height <= cluster_size {
            return;
        }

        let start_x = rng.random_range(0..self.width - cluster_size);
        let start_y = rng.random_range(0..self.height - cluster_size);

        // Spawn with a new random color
        let new_color = Color::srgb(rng.random(), rng.random(), rng.random());

        for y in 0..cluster_size {
            for x in 0..cluster_size {
                if rng.random_bool(self.genesis_density as f64) {
                    let idx = get_index(self.width, start_y + y, start_x + x);
                    self.cells[idx] = Some(new_color);
                }
            }
        }
    }

    /// Calculates and updates to the next generation.
    pub fn tick(&mut self) {
        // Check if it's time to spawn a new cluster of life
        if self.genesis_interval > 0 {
            self.tick_counter += 1;
            if self.tick_counter >= self.genesis_interval {
                self.tick_counter = 0;
                self.random_genesis();
            }
        }

        let width = self.width;
        let height = self.height;
        let cells = &self.cells;

        self.next_cells
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, next_cell)| {
                let y = index / width;
                let x = index % width;
                let (live_neighbors, neighbor_colors) =
                    get_live_neighbors_info(cells, width, height, y, x);
                let current_cell = cells[index];

                *next_cell = match (current_cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours dies (underpopulation).
                    (Some(_), n) if n < 2 => None,
                    // Rule 2: Any live cell with two or three live neighbours lives on.
                    (Some(color), 2) | (Some(color), 3) => Some(color),
                    // Rule 3: Any live cell with more than three live neighbours dies (overpopulation).
                    (Some(_), n) if n > 3 => None,
                    // Rule 4: Any dead cell with exactly three live neighbours becomes a live cell (reproduction).
                    (None, 3) => {
                        // 如果没有邻居，直接返回 None
                        if neighbor_colors.is_empty() {
                            None
                        } else {
                            // 1. 使用 u8 数组作为键来计数
                            let mut color_counts = HashMap::new();
                            for color in neighbor_colors {
                                *color_counts
                                    .entry(color.to_srgba().to_u8_array_no_alpha())
                                    .or_insert(0) += 1;
                            }

                            // 2. 找到出现次数最多的字节数组
                            color_counts
                                .into_iter()
                                .max_by_key(|&(_, count)| count)
                                // 3. 将胜出的字节数组转换回 Bevy Color
                                .map(|(key, _)| Color::srgb_u8(key[0], key[1], key[2]))
                        }
                    }
                    // All other cells remain in their current state (e.g., dead cell without 3 neighbors).
                    (otherwise, _) => otherwise,
                };
            });

        std::mem::swap(&mut self.cells, &mut self.next_cells);
    }
}
