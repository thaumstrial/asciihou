use bevy::prelude::*;
use asciihou::ascii_animation::AsciiAnimationPlugin;
struct Cell {
    ch: char,
    color: String, // "#rrggbb"
}
struct AsciiFrame {
    cells: Vec<Cell>, // width * height
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AsciiAnimationPlugin)
        .run();
}