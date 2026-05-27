use std::io;

use ratagotchi::game::Game;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| Game::default().run(terminal))
}