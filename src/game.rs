use std::{io, path::PathBuf, sync::mpsc, thread, time::Duration};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent},
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    config::{
        GAME_AUTOSAVE_FREQUENCY, RATAGOTCHI_ACTIVITY_DURATION, RATAGOTCHI_AGE_CONVERSION_RATE,
        RATAGOTCHI_DECAY_FACTOR_DISCIPLINE, RATAGOTCHI_DECAY_FACTOR_HAPPINESS,
        RATAGOTCHI_DECAY_FACTOR_HEALTH, RATAGOTCHI_DECAY_FACTOR_HUNGER, STAT_INCREASE_FOOD_CHEESE,
        STAT_INCREASE_FOOD_RATFOOD,
    },
    ui::{GameView, RenderStats},
};

#[derive(Debug, Default)]
pub struct Game {
    exit: bool,
    pub state: GameState,
    pub active_view: GameView,
    pub highlighted_tab: GameView,
    pub tick_progress: usize,
    pub ratagotchi: Ratagotchi,
    pub activity_ticks: usize,
    pub choice: Choice,
    pub stats_screen: RenderStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ratagotchi {
    pub hunger: f32,
    pub health: f32,
    pub happiness: f32,
    pub discipline: f32,
    pub weight: f32,
    pub birth: DateTime<Utc>,
    pub age: f32,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub enum GameState {
    #[default]
    CatchingUp,
    Running,
    Exiting,
}

#[derive(Debug, Default, PartialEq)]
pub enum Choice {
    #[default]
    Top,
    Bottom,
}

#[derive(Debug, PartialEq)]
pub enum ChoiceDirection {
    Up,
    Down,
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
                    terminal.draw(|frame| self.draw(frame))?;
                    self.catchup();
                    self.next_state(GameState::Running);
                }
                GameState::Running => {
                    match rx.recv().unwrap() {
                        Trigger::Input(key_event) => self.handle_key_event(key_event),
                        Trigger::Tick => {
                            self.update_stats();
                            self.tick_increase();
                        }
                    }
                    if self.tick_progress.is_multiple_of(GAME_AUTOSAVE_FREQUENCY) {
                        self.save();
                    }
                    terminal.draw(|frame| self.draw(frame))?;
                }
                GameState::Exiting => {
                    terminal.draw(|frame| self.draw(frame))?;
                    self.save();
                    thread::sleep(Duration::from_millis(1000));
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
            KeyCode::Enter => {
                if self.active_view != self.highlighted_tab {
                    self.change_view();
                } else {
                    self.do_action();
                }
            }
            KeyCode::Esc => self.set_view(GameView::Main),
            KeyCode::Up => self.change_choice(ChoiceDirection::Up),
            KeyCode::Down => self.change_choice(ChoiceDirection::Down),
            KeyCode::Right => {
                if self.activity_ticks == 0 {
                    self.next_view()
                }
            }
            KeyCode::Left => {
                if self.activity_ticks == 0 {
                    self.prev_view();
                }
            }
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

    fn set_view(&mut self, view: GameView) {
        self.active_view = view;
    }

    fn tick_increase(&mut self) {
        self.tick_progress += 1;
        self.activity_ticks = if self.activity_ticks > 0 {
            self.activity_ticks - 1
        } else {
            0
        };
    }

    // TODO: I just threw random factors here, it probably needs a second look
    pub fn tick_decay(&mut self, ticks: f32) {
        self.ratagotchi.hunger -= RATAGOTCHI_DECAY_FACTOR_HUNGER * ticks;
        self.ratagotchi.health -= RATAGOTCHI_DECAY_FACTOR_HEALTH * ticks;
        self.ratagotchi.happiness -=
            RATAGOTCHI_DECAY_FACTOR_HAPPINESS * (100. / self.ratagotchi.hunger) * ticks;
        self.ratagotchi.discipline -=
            RATAGOTCHI_DECAY_FACTOR_DISCIPLINE * (100. / self.ratagotchi.happiness) * ticks;
    }

    pub fn update_stats(&mut self) {
        self.tick_decay(1.0);
        self.ratagotchi.age =
            (Utc::now() - self.ratagotchi.birth).as_seconds_f32() / RATAGOTCHI_AGE_CONVERSION_RATE;
    }

    fn next_state(&mut self, state: GameState) {
        self.state = state;
    }

    fn change_choice(&mut self, direction: ChoiceDirection) {
        if self.active_view == GameView::Health {
            if direction == ChoiceDirection::Up {
                self.stats_screen = match self.stats_screen {
                    RenderStats::Stats => RenderStats::Happiness,
                    RenderStats::Discipline => RenderStats::Stats,
                    RenderStats::Hungry => RenderStats::Discipline,
                    RenderStats::Happiness => RenderStats::Hungry,
                };
            } else {
                self.stats_screen = match self.stats_screen {
                    RenderStats::Stats => RenderStats::Discipline,
                    RenderStats::Discipline => RenderStats::Hungry,
                    RenderStats::Hungry => RenderStats::Happiness,
                    RenderStats::Happiness => RenderStats::Stats,
                };
            }
        } else {
            self.choice = if self.choice == Choice::Top {
                Choice::Bottom
            } else {
                Choice::Top
            };
        }
    }

    fn do_action(&mut self) {
        if self.active_view == GameView::Feed {
            let hunger_restored = match self.choice {
                Choice::Top => STAT_INCREASE_FOOD_CHEESE,
                Choice::Bottom => STAT_INCREASE_FOOD_RATFOOD,
            };
            self.activity_ticks = RATAGOTCHI_ACTIVITY_DURATION;
            self.ratagotchi.hunger = (self.ratagotchi.hunger + hunger_restored).min(100.0);
        }
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

    fn save(&mut self) {
        match self.get_storage_folder() {
            Ok(f) => {
                self.ratagotchi.last_update = Utc::now();
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

    fn catchup(&mut self) {
        let time_delta = (Utc::now() - self.ratagotchi.last_update).as_seconds_f32();
        let ticks_delta = time_delta * 2.0;
        self.tick_decay(ticks_delta);
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
            hunger: 100.0,
            health: 100.0,
            happiness: 100.0,
            discipline: 100.0,
            weight: 1.0,
            birth: Utc::now(),
            age: 0.0,
            last_update: Utc::now(),
        }
    }
}
impl Ratagotchi {
    pub fn new() -> Self {
        Default::default()
    }
}
