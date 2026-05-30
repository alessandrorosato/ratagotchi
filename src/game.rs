use std::{io, path::PathBuf, sync::mpsc, thread, time::Duration};

use directories::ProjectDirs;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent},
    },
};
use serde::{Deserialize, Serialize};

use crate::ui::GameView;

#[derive(Debug, Default)]
pub struct Game {
    exit: bool,
    state: GameState,
    pub active_view: GameView,
    pub highlighted_tab: GameView,
    pub tick_progress: usize,
    pub ratagotchi: Ratagotchi,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Trigger>,
    ) -> io::Result<()> {
        while !self.exit {
            match self.state {
                GameState::CatchingUp => {
                    self.load();
                    // TODO: Do the actual catch-up, which means adding last tick's datetime and doing some calculations.
                    self.next_state(GameState::Running);
                }
                GameState::Running => {
                    match rx.recv().unwrap() {
                        Trigger::Input(key_event) => self.handle_key_event(key_event),
                        Trigger::Tick => {
                            self.tick_decay();
                            self.tick_increase();
                        }
                    }
                    terminal.draw(|frame| self.draw(frame))?;
                }
                GameState::Exiting => {
                    self.save();
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

    // Get application's data folder for savegames
    // Linux:   /home/alice/.config/ratagotchi
    // Windows: C:\Users\Alice\AppData\Roaming\shelakynn\ratagotchi
    // macOS:   /Users/Alice/Library/Application Support/shelakynn.ratagotchi
    fn get_storage_folder(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if let Some(proj_dirs) = ProjectDirs::from("", "shelakynn", "ratagotchi") {
            let storage_folder = proj_dirs.data_dir();
            if !storage_folder.to_path_buf().exists() {
                std::fs::create_dir_all(storage_folder).unwrap();
            }
            Ok(storage_folder.to_path_buf())
        } else {
            Err("Failed to get directories".to_string())?
        }
    }

    fn save(&self) {
        match self.get_storage_folder() {
            Ok(f) => {
                let full_path = f.join("savegame.json");
                let data = serde_json::to_vec(&self.ratagotchi).unwrap();
                let _ = std::fs::write(full_path, data).inspect_err(|e| {
                    log::error!("Error while writing save file: {}", e);
                });
            }
            Err(e) => {
                log::error!("Error while getting storage folder: {}", e);
            }
        }
    }
    fn load(&mut self) {
        match self.get_storage_folder() {
            Ok(f) => {
                let full_path = f.join("savegame.json");
                if let Ok(json_data) = std::fs::read(full_path) {
                    match serde_json::from_slice::<Ratagotchi>(&json_data) {
                        Ok(r) => self.ratagotchi = r,
                        Err(e) => log::error!("Error while reading save file: {}", e),
                    };
                }
            }
            Err(e) => {
                log::error!("Error while getting storage folder: {}", e);
            }
        }
    }
}

pub enum Trigger {
    Input(KeyEvent),
    Tick,
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
            weight: 1.,
        }
    }
}
impl Ratagotchi {
    pub fn new() -> Self {
        Default::default()
    }
}
