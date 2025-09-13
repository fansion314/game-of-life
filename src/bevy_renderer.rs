use crate::game::{self, Game};
// --- Bevy Renderer ---
use crate::Cli;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
struct CellSprite(usize); // Holds the index of the cell in the Game struct

#[derive(Resource)]
struct GameColors {
    background: Color,
}

pub fn run(cli: Cli) {
    let game_width = cli.width.unwrap_or(120);
    let game_height = cli.height.unwrap_or(80);
    let cell_size = cli.cell_size;

    let initial_cell_color = parse_color(&cli.cell_color).unwrap_or(Color::WHITE);
    let bg_color = parse_color(&cli.bg_color).unwrap_or(Color::BLACK);

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Game of Life".into(),
                    resolution: (
                        game_width as f32 * cell_size,
                        game_height as f32 * cell_size,
                    )
                        .into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(bg_color))
        .insert_resource(GameColors {
            background: bg_color,
        })
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
            1.0 / cli.fps,
        )))
        .insert_resource(Game::new(
            game_width,
            game_height,
            cell_size,
            cli.initial_density,
            initial_cell_color,
            cli.genesis_interval,
            cli.genesis_cluster_size,
            cli.genesis_density,
        ))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (game_tick, update_visuals).chain())
        .run();
}

fn parse_color(s: &str) -> std::result::Result<Color, ()> {
    let s_lower = s.to_lowercase();
    let named_color = match s_lower.as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::srgb_u8(255, 0, 0)),
        "green" => Some(Color::srgb_u8(0, 255, 0)),
        "blue" => Some(Color::srgb_u8(0, 0, 255)),
        "yellow" => Some(Color::srgb_u8(255, 255, 0)),
        "cyan" => Some(Color::srgb_u8(0, 255, 255)),
        "magenta" => Some(Color::srgb_u8(255, 0, 255)),
        "orange" => Some(Color::srgb_u8(255, 165, 0)),
        "purple" => Some(Color::srgb_u8(128, 0, 128)),
        "pink" => Some(Color::srgb_u8(255, 192, 203)),
        "navy" => Some(Color::srgb_u8(0, 0, 128)),
        _ => None,
    };
    if let Some(color) = named_color {
        return Ok(color);
    }

    let parts: Vec<&str> = s.split(',').collect();
    if (parts.len() == 3)
        && let (Ok(r), Ok(g), Ok(b)) = (
            parts[0].trim().parse::<u8>(),
            parts[1].trim().parse::<u8>(),
            parts[2].trim().parse::<u8>(),
        )
    {
        return Ok(Color::srgb_u8(r, g, b));
    }
    Err(())
}

fn setup(mut commands: Commands, game: Res<Game>) {
    commands.spawn(Camera2d);

    let game_width = game.width;
    let game_height = game.height;
    let cell_size = game.cell_size;

    let cell_sprite = Sprite {
        color: Color::BLACK, // Will be updated in the first frame
        custom_size: Some(Vec2::new(cell_size, cell_size)),
        ..default()
    };

    for y in 0..game_height {
        for x in 0..game_width {
            let index = game::get_index(game_width, y, x);
            commands.spawn((
                cell_sprite.clone(),
                Transform::from_xyz(
                    (x as f32 - game_width as f32 / 2.0) * cell_size,
                    (y as f32 - game_height as f32 / 2.0) * cell_size,
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

fn update_visuals(
    game: Res<Game>,
    colors: Res<GameColors>,
    mut query: Query<(&mut Sprite, &CellSprite)>,
) {
    // This is now much more powerful, as it can render any color.
    for (mut sprite, cell) in query.iter_mut() {
        sprite.color = match game.cells[cell.0] {
            Some(cell_color) => cell_color, // Use the cell's actual color
            None => colors.background,      // Use the background color if dead
        };
    }
}
