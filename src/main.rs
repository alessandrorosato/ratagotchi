use std::{io, sync::mpsc, thread};

use ratagotchi::game::{Game, Trigger, handle_input_events, tick_lapse};

fn main() -> io::Result<()> {
    let (trigger_tx, trigger_rx) = mpsc::channel::<Trigger>();
    let tx_to_input_events = trigger_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_tick = trigger_tx.clone();
    thread::spawn(move || {
        tick_lapse(tx_to_tick);
    });

    ratatui::run(|terminal| Game::default().run(terminal, trigger_rx))
}
