use std::{io, sync::mpsc, thread, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{self, event::{Event, KeyCode, KeyEvent}},
};

use crate::ui::GameView;

#[derive(Debug, Default)]
pub struct Game {
    exit: bool,
    state: GameState,
    pub active_view: GameView,
    pub highlighted_tab: GameView,
    pub tick_progress: usize,
    pub ratagotchi: Ratagotchi
}

#[derive(Debug)]
pub struct Ratagotchi {
    pub hunger: f32,
    pub health: f32,
    pub happiness: f32,
    pub discipline: f32,
    pub weight: f32,
}

#[derive(Debug, Default)]
pub enum GameState {
    #[default]
    CatchingUp,
    Running,
    Exiting,
}

impl Game {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Trigger>) -> io::Result<()> {
        while !self.exit {
            match self.state {
                GameState::CatchingUp => {
                    // TODO: Here we supposedly load the savegame and simulate the time diff
                    self.next_state(GameState::Running);
                }
                GameState::Running => {
                    match rx.recv().unwrap() {
                        Trigger::Input(key_event) => self.handle_key_event(key_event),
                        Trigger::Tick => {
                            self.tick_decay();
                            self.tick_increase();
                        },
                    }
                    terminal.draw(|frame| self.draw(frame))?;
                },
                GameState::Exiting => {
                    // TODO: Save game state before exiting
                    self.exit();
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.next_state(GameState::Exiting),
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

    fn tick_increase(&mut self) {
        self.tick_progress += 1;
    }
    
    // TODO: I just threw random factors here, it probably needs a second look
    pub fn tick_decay(&mut self) {
        self.ratagotchi.hunger -= 0.004;
        self.ratagotchi.health -= 0.001;
        self.ratagotchi.happiness -= 0.002 * (100. / self.ratagotchi.hunger);
        self.ratagotchi.discipline -= 0.004 * (100. / self.ratagotchi.happiness);
    }

    fn next_state(&mut self, state: GameState) {
        self.state = state;
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

impl Default for Ratagotchi {
    fn default() -> Self {
        Ratagotchi {
            hunger: 100.,
            health: 100.,
            happiness: 100.,
            discipline: 100.,
            weight: 1.
        }
    }
}
impl Ratagotchi {
    pub fn new() -> Self {
        Default::default()
    }
}