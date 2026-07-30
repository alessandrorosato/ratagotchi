use std::{io, sync::mpsc, thread};

use log::info;
use log4rs::{
    Config,
    append::file::FileAppender,
    config::{Appender, Root},
};
use ratagotchi::{
    game::{Game, Trigger, handle_input_events, tick_lapse},
    utils::get_storage_folder,
};

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

    let log_path = get_storage_folder().unwrap().join("ratagotchi.log");

    let appender = FileAppender::builder().build(log_path)?;
    let log_config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(appender)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(log::LevelFilter::Info),
        );

    if let Ok(l) = log_config
        && log4rs::init_config(l).is_err()
    {
        panic!("Can't initialise logger.");
    }

    info!("Starting terminal interface.");
    ratatui::run(|terminal| Game::default().run(terminal, trigger_rx))
}
