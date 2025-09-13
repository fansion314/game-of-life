use crate::game::Game;
// --- Terminal Renderer ---
use crate::Cli;
use crossterm::{
    cursor, event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use image::{DynamicImage, GrayImage, Luma};
use std::io::stdout;
use std::time::{Duration, Instant};
use viuer::{print, Config};

/// Renders the game state to an image buffer.
fn render_to_image(game: &Game, pixel_scale: u32) -> GrayImage {
    let img_width = game.width as u32 * pixel_scale;
    let img_height = game.height as u32 * pixel_scale;
    let mut img = GrayImage::new(img_width, img_height);

    for y in 0..game.height {
        for x in 0..game.width {
            let index = y * game.width + x;
            // Render as white if the cell is Some(color), black if None
            let pixel = if game.cells[index].is_some() {
                Luma([255u8]) // White
            } else {
                Luma([0u8]) // Black
            };
            for dy in 0..pixel_scale {
                for dx in 0..pixel_scale {
                    img.put_pixel(
                        (x as u32 * pixel_scale) + dx,
                        (y as u32 * pixel_scale) + dy,
                        pixel,
                    );
                }
            }
        }
    }
    img
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let (term_cols, term_rows) = terminal::size()?;
    let game_width = cli.width.unwrap_or(term_cols as usize);
    let game_height = cli.height.unwrap_or((term_rows * 2) as usize);

    // Terminal renderer uses a fixed white color for initial cells
    let initial_color = bevy::prelude::Color::WHITE;

    let mut game = Game::new(
        game_width,
        game_height,
        cli.cell_size,
        cli.initial_density,
        initial_color,
        cli.genesis_interval,
        cli.genesis_cluster_size,
        cli.genesis_density,
    );

    let frame_duration = Duration::from_secs_f64(1.0 / cli.fps as f64);

    loop {
        let frame_start = Instant::now();

        if (event::poll(Duration::from_millis(0))?)
            && let Event::Key(key) = event::read()?
            && (key.code == KeyCode::Char('q') || key.code == KeyCode::Esc)
        {
            break;
        }

        game.tick();

        let image = render_to_image(&game, cli.pixel_scale);
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
