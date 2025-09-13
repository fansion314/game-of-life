//! Conway's Game of Life, with two rendering backends.

// --- Core Game Logic (backend-agnostic) ---
mod game {
    use bevy::prelude::Resource;
    use rand::Rng;
    use rayon::prelude::*;

    /// 将二维坐标转换为一维向量的索引
    pub fn get_index(width: usize, row: usize, column: usize) -> usize {
        row * width + column
    }

    /// 计算给定细胞周围的活邻居数量 (环形边界)
    fn live_neighbor_count(
        cells: &[bool],
        width: usize,
        height: usize,
        row: usize,
        column: usize,
    ) -> u8 {
        let mut count = 0;
        for delta_row in [height - 1, 0, 1].iter() {
            for delta_col in [width - 1, 0, 1].iter() {
                if *delta_row == 0 && *delta_col == 0 {
                    continue;
                }
                let neighbor_row = (row + delta_row) % height;
                let neighbor_col = (column + delta_col) % width;
                let idx = get_index(width, neighbor_row, neighbor_col);
                if cells[idx] {
                    count += 1;
                }
            }
        }
        count
    }

    /// 代表游戏世界
    #[derive(Resource)]
    pub struct Game {
        pub width: usize,
        pub height: usize,
        pub cells: Vec<bool>,
        next_cells: Vec<bool>,
    }

    impl Game {
        /// 创建一个新的游戏实例
        pub fn new(width: usize, height: usize) -> Game {
            let size = width * height;
            let mut cells = vec![false; size];
            let mut rng = rand::rng();
            for cell in cells.iter_mut() {
                *cell = rng.random();
            }

            Game {
                width,
                height,
                cells,
                next_cells: vec![false; size],
            }
        }

        /// 计算并更新到下一轮的游戏状态
        pub fn tick(&mut self) {
            let width = self.width;
            let height = self.height;
            let cells = &self.cells;

            self.next_cells
                .par_iter_mut()
                .enumerate()
                .for_each(|(index, next_cell)| {
                    let y = index / width;
                    let x = index % width;
                    let live_neighbors = live_neighbor_count(cells, width, height, y, x);
                    let current_cell = cells[index];

                    *next_cell = match (current_cell, live_neighbors) {
                        (true, n) if n < 2 => false,
                        (true, 2) | (true, 3) => true,
                        (true, n) if n > 3 => false,
                        (false, 3) => true,
                        (otherwise, _) => otherwise,
                    };
                });

            std::mem::swap(&mut self.cells, &mut self.next_cells);
        }
    }
}

// --- Terminal Renderer ---
mod terminal_renderer {
    use crate::game::Game;
    use crossterm::{
        cursor, event::{self, Event, KeyCode},
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    };
    use image::{DynamicImage, GrayImage, Luma};
    use std::io::{stdout, Result};
    use std::time::{Duration, Instant};
    use viuer::{print, Config};

    const PIXEL_SCALE: u32 = 2; // 每个细胞渲染的像素大小 (2x2)

    /// 将游戏状态渲染为图像缓冲区
    fn render_to_image(game: &Game) -> GrayImage {
        let img_width = game.width as u32 * PIXEL_SCALE;
        let img_height = game.height as u32 * PIXEL_SCALE;
        let mut img = GrayImage::new(img_width, img_height);

        for y in 0..game.height {
            for x in 0..game.width {
                let index = y * game.width + x;
                let pixel = if game.cells[index] {
                    Luma([255u8]) // 白色
                } else {
                    Luma([0u8]) // 黑色
                };
                for dy in 0..PIXEL_SCALE {
                    for dx in 0..PIXEL_SCALE {
                        img.put_pixel(
                            (x as u32 * PIXEL_SCALE) + dx,
                            (y as u32 * PIXEL_SCALE) + dy,
                            pixel,
                        );
                    }
                }
            }
        }
        img
    }

    pub fn run() -> Result<()> {
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(cursor::Hide)?;
        terminal::enable_raw_mode()?;

        let (term_cols, term_rows) = terminal::size()?;
        let game_width = term_cols as usize;
        let game_height = (term_rows * 2) as usize;
        let mut game = Game::new(game_width, game_height);

        let frame_duration = Duration::from_secs_f64(1.0 / 60.0);

        loop {
            let frame_start = Instant::now();

            if (event::poll(Duration::from_millis(0))?)
                && let Event::Key(key) = event::read()?
                && (key.code == KeyCode::Char('q') || key.code == KeyCode::Esc)
            {
                break;
            }

            game.tick();

            let image = render_to_image(&game);
            let dynamic_image = DynamicImage::ImageLuma8(image);
            let conf = Config {
                x: 0,
                y: 0,
                ..Default::default()
            };
            print(&dynamic_image, &conf).expect("Image printing failed.");

            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }

        terminal::disable_raw_mode()?;
        stdout.execute(cursor::Show)?;
        stdout.execute(LeaveAlternateScreen)?;
        Ok(())
    }
}

// --- Bevy Renderer ---
mod bevy_renderer {
    use crate::game;
    use crate::game::Game;
    use bevy::prelude::*;
    use std::time::Duration;

    const GAME_WIDTH: usize = 120;
    const GAME_HEIGHT: usize = 80;
    const CELL_SIZE: f32 = 8.0;

    #[derive(Component)]
    struct CellSprite(usize); // Holds the index of the cell in the Game struct

    pub fn run() {
        App::new()
            .add_plugins(
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Game of Life".into(),
                        resolution: (
                            GAME_WIDTH as f32 * CELL_SIZE,
                            GAME_HEIGHT as f32 * CELL_SIZE,
                        )
                            .into(),
                        ..default()
                    }),
                    ..default()
                }),
            )
            .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
                1.0 / 120.0,
            )))
            .insert_resource(Game::new(GAME_WIDTH, GAME_HEIGHT))
            .add_systems(Startup, setup)
            .add_systems(FixedUpdate, (game_tick, update_visuals).chain())
            .run();
    }

    fn setup(mut commands: Commands, game: Res<Game>) {
        commands.spawn(Camera2d);

        let cell_sprite = Sprite {
            color: Color::BLACK,
            custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
            ..default()
        };

        for y in 0..game.height {
            for x in 0..game.width {
                let index = game::get_index(game.width, y, x);
                commands.spawn((
                    cell_sprite.clone(),
                    Transform::from_xyz(
                        (x as f32 - GAME_WIDTH as f32 / 2.0) * CELL_SIZE,
                        (y as f32 - GAME_HEIGHT as f32 / 2.0) * CELL_SIZE,
                        0.0,
                    ),
                    CellSprite(index),
                ));
            }
        }
    }

    fn game_tick(mut game: ResMut<Game>) {
        game.tick();
    }

    fn update_visuals(game: Res<Game>, mut query: Query<(&mut Sprite, &CellSprite)>) {
        for (mut sprite, cell) in query.iter_mut() {
            sprite.color = if game.cells[cell.0] {
                Color::WHITE
            } else {
                Color::BLACK
            };
        }
    }
}

// --- Main App ---
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Choose the rendering backend
    #[arg(short, long, value_enum, default_value_t = Renderer::Terminal)]
    renderer: Renderer,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Renderer {
    /// Render in the terminal using viuer
    Terminal,
    /// Render in a 2D window using Bevy
    Bevy,
}

fn main() {
    let cli = Cli::parse();

    match cli.renderer {
        Renderer::Terminal => {
            if let Err(e) = terminal_renderer::run() {
                eprintln!("Terminal renderer error: {}", e);
            }
        }
        Renderer::Bevy => {
            bevy_renderer::run();
        }
    }
}
