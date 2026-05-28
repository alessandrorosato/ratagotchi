use std::{io, sync::mpsc, thread, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{self, event::{Event, KeyCode, KeyEvent}},
};

use crate::ui::GameView;

#[derive(Debug, Default)]
pub struct Game {
    exit: bool,
    pub active_view: GameView,
    pub highlighted_tab: GameView,
    pub tick_progress: usize,
}

pub enum GameState {
    CatchingUp,
    Running,
    Exiting,
}

impl Game {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Trigger>) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Trigger::Input(key_event) => self.handle_key_event(key_event),
                Trigger::Tick => self.tick_progress += 1,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Tab => self.next_view(),
            KeyCode::BackTab => self.prev_view(),
            KeyCode::Enter => self.change_view(),
            _ => {}
        }
    }

    fn next_view(&mut self) {
        self.highlighted_tab = match self.highlighted_tab {
            GameView::Main => GameView::Feed,
            GameView::Feed => GameView::Lights,
            GameView::Lights => GameView::Play,
            GameView::Play => GameView::Medicine,
            GameView::Medicine => GameView::Bath,
            GameView::Bath => GameView::Health,
            GameView::Health => GameView::Scold,
            GameView::Scold => GameView::Attention,
            GameView::Attention => GameView::Main,
        };
    }

    fn prev_view(&mut self) {
        self.highlighted_tab = match self.highlighted_tab {
            GameView::Main => GameView::Attention,
            GameView::Feed => GameView::Main,
            GameView::Lights => GameView::Feed,
            GameView::Play => GameView::Lights,
            GameView::Medicine => GameView::Play,
            GameView::Bath => GameView::Medicine,
            GameView::Health => GameView::Bath,
            GameView::Scold => GameView::Health,
            GameView::Attention => GameView::Scold,
        };
    }

    fn change_view(&mut self) {
        self.active_view = self.highlighted_tab.clone();
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

pub enum Trigger {
    Input(KeyEvent),
    Tick
}

pub fn handle_input_events(tx: mpsc::Sender<Trigger>) {
    loop {
        if let Event::Key(key_event) = crossterm::event::read().unwrap() {
            tx.send(Trigger::Input(key_event)).unwrap()
        }
    }
}

pub fn tick_lapse(tx: mpsc::Sender<Trigger>) {
    loop {
        thread::sleep(Duration::from_millis(500));
        tx.send(Trigger::Tick).unwrap();
    }
}