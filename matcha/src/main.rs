mod game_state;
mod piece;
mod animation;
mod drawable;
mod matcha_board;

use cgmath::Vector2;
use crate::game_state::GameState;

fn main() {
    let size = (1000, 800);
    let mut rng = rand::rng();
    let game_state = GameState::new(&mut rng, size);

    let _ = pollster::block_on(bananagraph::run_window("Matcha!", size.into(), Vector2::from(size) / 4, game_state));
}
