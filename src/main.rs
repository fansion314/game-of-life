//! Conway's Game of Life, with two rendering backends and advanced color genetics.

// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bevy_renderer;
mod game;
mod terminal_renderer;

// --- Main App ---
use clap::{Parser, ValueEnum};

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Choose the rendering backend
    #[arg(short, long, value_enum, default_value_t = Renderer::Bevy)]
    renderer: Renderer,

    /// Width of the game grid
    #[arg(long)]
    width: Option<usize>,

    /// Height of the game grid
    #[arg(long)]
    height: Option<usize>,

    /// Target frames per second (tick rate)
    #[arg(long, default_value_t = 60.0)]
    fps: f32,

    /// [Bevy] Color of the initial living cells. E.g., "red" or "0,255,127"
    #[arg(long, default_value = "white")]
    cell_color: String,

    /// [Bevy] Color of the background. E.g., "navy" or "20,20,40"
    #[arg(long, default_value = "black")]
    bg_color: String,

    // --- New Hyperparameters ---
    /// Initial density of living cells (0.0 to 1.0)
    #[arg(long, default_value_t = 0.5)]
    initial_density: f32,

    /// Spawn a new random life cluster every N ticks. 0 disables.
    #[arg(long, default_value_t = 300)]
    genesis_interval: u32,

    /// The size of the square for the new life cluster (NxN)
    #[arg(long, default_value_t = 10)]
    genesis_cluster_size: u32,

    /// The density of life within the new cluster (0.0 to 1.0)
    #[arg(long, default_value_t = 0.6)]
    genesis_density: f32,

    /// [Bevy] The size of each cell in pixels
    #[arg(long, default_value_t = 8.0)]
    cell_size: f32,

    /// [Terminal] The scale of each cell in pixels (e.g., 2 means 2x2)
    #[arg(long, default_value_t = 2)]
    pixel_scale: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Renderer {
    /// Render in the terminal using viuer (monochrome)
    Terminal,
    /// Render in a 2D window using Bevy (supports color)
    Bevy,
}

fn main() {
    let cli = Cli::parse();

    match cli.renderer {
        Renderer::Terminal => {
            println!("Starting terminal renderer... Press 'q' or 'Esc' to quit.");
            if let Err(e) = terminal_renderer::run(cli) {
                eprintln!("Terminal renderer error: {}", e);
            }
        }
        Renderer::Bevy => {
            // We need to insert cli as a NonSend resource for Bevy setup
            bevy_renderer::run(cli);
        }
    }
}
