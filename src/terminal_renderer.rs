// --- 终端渲染器 ---
use crate::bevy_renderer::parse_color;
use crate::game::Game;
use crate::{game, Cli};
use bevy::prelude::ColorToPacked;
use crossterm::{
    cursor, event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use image::{DynamicImage, Rgb, RgbImage};
use std::io::stdout;
use std::time::{Duration, Instant};
use viuer::{print, Config};

/// 将游戏状态渲染到一个彩色的图像缓冲区。
// 改变 2: 函数的返回类型从 GrayImage 变为 RgbImage
fn render_to_image(game: &Game, pixel_scale: u32) -> RgbImage {
    let img_width = game.width as u32 * pixel_scale;
    let img_height = game.height as u32 * pixel_scale;
    // 改变 3: 创建一个新的 RgbImage 而不是 GrayImage
    let mut img = RgbImage::new(img_width, img_height);

    for y in 0..game.height {
        for x in 0..game.width {
            let index = game::get_index(game.width, y, x);

            // 改变 4: 核心逻辑 - 将细胞颜色转换为像素颜色
            // 如果细胞存活 (Some(color))，则将其 Bevy Color 转换为 Rgb<u8> 像素。
            // Bevy Color 的各通道是 0.0 到 1.0 之间的 f32，我们需要将其映射到 0 到 255 的 u8。
            // 如果细胞死亡 (None)，则使用黑色像素。
            let pixel = if let Some(color) = game.cells[index] {
                Rgb(color.to_srgba().to_u8_array_no_alpha())
            } else {
                Rgb([0u8, 0, 0]) // 黑色
            };

            // 用计算出的像素颜色填充放大后的方块
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
    // 考虑到终端字符通常是长方形的，乘以2可以得到一个更接近方形的渲染区域
    let game_height = cli.height.unwrap_or((term_rows * 2) as usize);

    // 终端渲染器使用固定的白色作为初始细胞颜色
    let initial_color = parse_color(&cli.cell_color).unwrap_or(bevy::prelude::Color::WHITE);

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

        // 处理退出事件
        if (event::poll(Duration::from_millis(0))?)
            && let Event::Key(key) = event::read()?
            && (key.code == KeyCode::Char('q') || key.code == KeyCode::Esc)
        {
            break;
        }

        game.tick();

        let image = render_to_image(&game, cli.pixel_scale);
        // 改变 5: 将 RgbImage 包装成 DynamicImage::ImageRgb8
        let dynamic_image = DynamicImage::ImageRgb8(image);

        // 改变 6: 在 viuer 配置中显式启用真彩色以获得最佳效果
        let conf = Config {
            x: 0,
            y: 0,
            truecolor: true,
            ..Default::default()
        };
        print(&dynamic_image, &conf).expect("Image printing failed.");

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    // 清理并退出
    terminal::disable_raw_mode()?;
    stdout.execute(cursor::Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}
