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
        GAME_AUTOSAVE_FREQUENCY, RATAGOTCHI_ACTIVITY_DURATION, RATAGOTCHI_BIRTH_WEIGHT_RANGE,
        RATAGOTCHI_DECAY_FACTOR_DISCIPLINE, RATAGOTCHI_DECAY_FACTOR_HAPPINESS,
        RATAGOTCHI_DECAY_FACTOR_HEALTH, RATAGOTCHI_DECAY_FACTOR_HUNGER,
        RATAGOTCHI_DECAY_FACTOR_POOP, RATAGOTCHI_FITNESS_CHANGE_L, RATAGOTCHI_FITNESS_CHANGE_S,
        RATAGOTCHI_FITNESS_DECAY_INTERVAL, RATAGOTCHI_STAT_MAX_VALUE, RATAGOTCHI_STAT_MIN_VALUE,
        SECONDS_PER_DAY, STAT_INCREASE_FOOD_CHEESE, STAT_INCREASE_FOOD_RATFOOD,
    },
    minigame::{MiniGame, MiniGamePick},
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
    pub is_action_over: bool,
    pub is_dark: bool,
    pub mini_game: MiniGame,
    is_mini_game_running: bool,
    pub main_menu_index: MainMenuIndex,
    pub main_menu_popup: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ratagotchi {
    pub hunger: f32,
    pub health: f32,
    pub happiness: f32,
    pub discipline: f32,
    birth_weight: f32,
    pub weight: f32,
    fitness: Fitness,
    pub poop: f32,
    pub birth: DateTime<Utc>,
    pub age: f32,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Default, PartialEq)]
pub enum GameState {
    #[default]
    Menu,
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

#[derive(Debug, Serialize, Deserialize)]
struct Fitness {
    value: f32,
    last_update: DateTime<Utc>,
}

struct FitnessChange(f32);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MainMenuIndex(pub usize);

impl Game {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Trigger>,
    ) -> io::Result<()> {
        while !self.exit {
            match self.state {
                GameState::Menu => {
                    match rx.recv() {
                        Ok(trigger) => {
                            if let Trigger::Input(key_event) = trigger {
                                self.handle_key_event(key_event)
                            }
                        }
                        Err(_) => {
                            // TODO: logging
                            println!("Receiver error.");
                            break;
                        }
                    }
                    terminal.draw(|frame| self.draw(frame))?;
                }
                GameState::CatchingUp => {
                    self.load();
                    terminal.draw(|frame| self.draw(frame))?;
                    self.catchup();
                    self.next_state(GameState::Running);
                }
                GameState::Running => {
                    match rx.recv() {
                        Ok(trigger) => match trigger {
                            Trigger::Input(key_event) => self.handle_key_event(key_event),
                            Trigger::Tick => {
                                self.update_stats();
                                self.ratagotchi.fitness_decay(
                                    (self.ratagotchi.fitness.last_update - Utc::now())
                                        .as_seconds_f32(),
                                );
                                self.tick_increase();
                                self.back_to_main();
                            }
                        },
                        Err(_) => {
                            // TODO: logging
                            println!("Receiver error.");
                            break;
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
        if self.state == GameState::Menu {
            match key_event.code {
                KeyCode::Up => self.main_menu_index_decrease(),
                KeyCode::Down => self.main_menu_index_increase(),
                KeyCode::Enter => self.main_menu_action(),
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Esc => self.main_menu_popup = false,
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Char('q') => self.next_state(GameState::Exiting),
                KeyCode::Tab => self.next_view(),
                KeyCode::BackTab => self.prev_view(),
                KeyCode::Enter => {
                    if self.jump_to_action() {
                        self.do_action();
                    } else if self.highlighted_tab == GameView::Lights {
                        self.is_dark = !self.is_dark;
                    } else if self.active_view != self.highlighted_tab {
                        self.change_view();
                    } else {
                        self.do_action();
                    }
                }
                KeyCode::Esc => {
                    self.set_view(GameView::Main);
                    self.highlighted_tab = GameView::Main;
                }
                KeyCode::Up => self.change_choice(ChoiceDirection::Up),
                KeyCode::Down => self.change_choice(ChoiceDirection::Down),
                KeyCode::Right => {
                    if self.is_mini_game_running {
                        self.change_minigame_pick();
                    } else if self.activity_ticks == 0 {
                        self.next_view()
                    }
                }
                KeyCode::Left => {
                    if self.is_mini_game_running {
                        self.change_minigame_pick();
                    } else if self.activity_ticks == 0 {
                        self.prev_view();
                    }
                }
                _ => {}
            }
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
            GameView::Attention => GameView::Feed,
        };
    }

    fn prev_view(&mut self) {
        self.highlighted_tab = match self.highlighted_tab {
            GameView::Main => GameView::Attention,
            GameView::Feed => GameView::Attention,
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
        let previous_activity_tick = self.activity_ticks;
        self.tick_progress += 1;
        self.activity_ticks = if self.activity_ticks > 0 {
            self.activity_ticks - 1
        } else {
            0
        };
        if previous_activity_tick == 1 {
            self.is_action_over = true;
        }
        if self.is_mini_game_running
            && let Some(result) = self.mini_game.run()
        {
            self.ratagotchi.happiness += result;
            self.is_mini_game_running = false;
            self.set_view(GameView::Main);
        }
    }

    fn jump_to_action(&self) -> bool {
        if matches!(
            self.highlighted_tab,
            GameView::Bath | GameView::Medicine | GameView::Scold | GameView::Play
        ) {
            return true;
        }
        false
    }

    // TODO: I just threw random factors here, it probably needs a second look
    pub fn tick_decay(&mut self, ticks: f32) {
        self.ratagotchi.hunger = (self.ratagotchi.hunger - RATAGOTCHI_DECAY_FACTOR_HUNGER * ticks)
            .max(RATAGOTCHI_STAT_MIN_VALUE);
        self.ratagotchi.health = (self.ratagotchi.health - RATAGOTCHI_DECAY_FACTOR_HEALTH * ticks)
            .max(RATAGOTCHI_STAT_MIN_VALUE);
        self.ratagotchi.happiness = (self.ratagotchi.happiness
            - RATAGOTCHI_DECAY_FACTOR_HAPPINESS * (100. / self.ratagotchi.hunger).min(50.) * ticks)
            .max(RATAGOTCHI_STAT_MIN_VALUE);
        self.ratagotchi.discipline = (self.ratagotchi.discipline
            - RATAGOTCHI_DECAY_FACTOR_DISCIPLINE
                * (100. / self.ratagotchi.happiness).min(50.)
                * ticks)
            .max(RATAGOTCHI_STAT_MIN_VALUE);
        self.ratagotchi.poop = (self.ratagotchi.poop + RATAGOTCHI_DECAY_FACTOR_POOP * ticks)
            .min(RATAGOTCHI_STAT_MAX_VALUE);
    }

    pub fn update_stats(&mut self) {
        self.tick_decay(1.0);
        let real_secs_delta = (Utc::now() - self.ratagotchi.birth).as_seconds_f32();
        self.ratagotchi.age = Self::convert_secs_to_game_days(real_secs_delta);
        self.ratagotchi.update_weight();
    }

    // Conversion (real to simulated):
    // * 8 hrs  -> 1 month
    // * 4 days -> 1 year
    pub fn convert_secs_to_game_days(real_secs: f32) -> f32 {
        let days_per_second = 90.0 / SECONDS_PER_DAY;
        real_secs * days_per_second
    }

    // TODO: This should change based on the ratagotchi health, fitness (to be implemented) and illness (to be implemented)
    // The lower stats, the less efficient actions are (i.e. you need to stay on top of it if it's in a bad shape)
    pub fn bonus_coefficient(&mut self) -> f32 {
        1.0
    }

    // TODO: similarly to the above, "malus_coefficient" for decays and other similar effects
    // The lower stats, the faster stats will decay (i.e. keeping the rat sick will precipitate the situation)
    pub fn malus_coefficient(&mut self) -> f32 {
        1.0
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

    fn back_to_main(&mut self) {
        if self.is_action_over && self.activity_ticks == 0 {
            self.active_view = GameView::default();
            self.highlighted_tab = GameView::default();
            self.is_action_over = false;
        }
    }

    fn do_action(&mut self) {
        let bonus = self.bonus_coefficient();
        if self.active_view == GameView::Feed {
            let (hunger_restored, happiness_restored, fitness_variation) = match self.choice {
                Choice::Top => {
                    let fitness_variation = if self.ratagotchi.hunger > 95.0 {
                        FitnessChange(RATAGOTCHI_FITNESS_CHANGE_L)
                    } else {
                        FitnessChange(RATAGOTCHI_FITNESS_CHANGE_S)
                    };
                    (STAT_INCREASE_FOOD_CHEESE, 10.0 * bonus, fitness_variation)
                }
                Choice::Bottom => {
                    let fitness_variation = if self.ratagotchi.hunger > 90.0 {
                        FitnessChange(RATAGOTCHI_FITNESS_CHANGE_S)
                    } else {
                        FitnessChange(0.0)
                    };
                    (STAT_INCREASE_FOOD_RATFOOD, 5.0 * bonus, fitness_variation)
                }
            };
            self.activity_ticks = RATAGOTCHI_ACTIVITY_DURATION;
            self.ratagotchi.hunger =
                (self.ratagotchi.hunger + hunger_restored).min(RATAGOTCHI_STAT_MAX_VALUE);
            self.ratagotchi.happiness =
                (self.ratagotchi.happiness + happiness_restored).min(RATAGOTCHI_STAT_MAX_VALUE);
            self.ratagotchi.update_fitness(fitness_variation, 1);
        } else if self.highlighted_tab == GameView::Bath && self.active_view != GameView::Bath {
            self.active_view = GameView::Bath;
            self.activity_ticks = RATAGOTCHI_ACTIVITY_DURATION;
            self.ratagotchi.poop = RATAGOTCHI_STAT_MIN_VALUE;
            let happiness_restored = if self.ratagotchi.poop > 50.0 {
                25.0 * bonus
            } else if self.ratagotchi.poop > 10.0 {
                15.0 * bonus
            } else {
                0.0
            };
            self.ratagotchi.happiness =
                (self.ratagotchi.happiness + happiness_restored).min(RATAGOTCHI_STAT_MAX_VALUE)
        } else if self.highlighted_tab == GameView::Medicine
            && self.active_view != GameView::Medicine
        {
            self.active_view = GameView::Medicine;
            self.activity_ticks = RATAGOTCHI_ACTIVITY_DURATION;
            self.ratagotchi.health += 60.0;
        } else if self.highlighted_tab == GameView::Scold && self.active_view != GameView::Scold {
            self.active_view = GameView::Scold;
            self.activity_ticks = RATAGOTCHI_ACTIVITY_DURATION;
            self.ratagotchi.discipline += 40.0;
        } else if self.highlighted_tab == GameView::Play
            && self.active_view != GameView::Play
            && !self.is_mini_game_running
        {
            self.mini_game = MiniGame::new();
            self.active_view = GameView::Play;
            self.is_mini_game_running = true;
        }
    }

    fn change_minigame_pick(&mut self) {
        self.mini_game.current_pick = if self.mini_game.current_pick == MiniGamePick::Higher {
            MiniGamePick::Lower
        } else {
            MiniGamePick::Higher
        };
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
                std::fs::create_dir_all(storage_folder)?;
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
                match serde_json::to_vec(&self.ratagotchi) {
                    Ok(data) => {
                        let _ = std::fs::write(full_path, data).inspect_err(|e| {
                            log::error!("Error while writing save file: {}", e);
                        });
                    }
                    Err(e) => log::error!("Error while getting storage folder: {}", e),
                };
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
        self.ratagotchi.fitness_decay(time_delta);
    }

    fn main_menu_index_increase(&mut self) {
        if !self.main_menu_popup {
            let current_index = self.main_menu_index.0;
            let new_index = (current_index + 1).min(2);
            self.main_menu_index = MainMenuIndex(new_index);
        }
    }

    fn main_menu_index_decrease(&mut self) {
        if !self.main_menu_popup {
            let current_index = self.main_menu_index.0;
            if current_index == 0 {
                return;
            }
            let new_index = (current_index - 1).max(0);
            self.main_menu_index = MainMenuIndex(new_index);
        }
    }

    fn main_menu_action(&mut self) {
        match self.main_menu_index.0 {
            0 => self.next_state(GameState::CatchingUp),
            1 => {
                if self.main_menu_popup {
                    self.next_state(GameState::Running)
                } else {
                    self.main_menu_popup = true
                }
            }
            2 => self.exit = true,
            _ => {}
        }
    }
}

pub enum Trigger {
    Input(KeyEvent),
    Tick,
}

pub fn handle_input_events(tx: mpsc::Sender<Trigger>) {
    loop {
        match crossterm::event::read() {
            Ok(Event::Key(key_event)) => {
                if let Err(e) = tx.send(Trigger::Input(key_event)) {
                    log::error!("Failed to transmit key event: {}", e);
                    break;
                }
            }
            Ok(_) => {} // Discard all the other event types
            Err(e) => {
                log::error!("Failed to read terminal event: {}", e);
            }
        }
    }
}

pub fn tick_lapse(tx: mpsc::Sender<Trigger>) {
    loop {
        thread::sleep(Duration::from_millis(500));
        if let Err(e) = tx.send(Trigger::Tick) {
            log::error!("Failed to transmit tick: {}", e);
            break;
        }
    }
}

impl Default for Ratagotchi {
    fn default() -> Self {
        let birth_weight = Self::starting_weight();
        Ratagotchi {
            hunger: RATAGOTCHI_STAT_MAX_VALUE,
            health: RATAGOTCHI_STAT_MAX_VALUE,
            happiness: RATAGOTCHI_STAT_MAX_VALUE,
            discipline: RATAGOTCHI_STAT_MAX_VALUE,
            birth_weight,
            weight: birth_weight,
            fitness: Fitness {
                value: 1.0,
                last_update: Utc::now(),
            },
            poop: RATAGOTCHI_STAT_MIN_VALUE,
            birth: Utc::now(),
            age: RATAGOTCHI_STAT_MIN_VALUE,
            last_update: Utc::now(),
        }
    }
}

impl Ratagotchi {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn print_age(&self) -> String {
        if self.age < 1.0 {
            "newborn".to_string()
        } else if self.age < 30.0 {
            format!(
                "{} day{}",
                self.age.floor(),
                if self.age >= 2.0 { "s" } else { "" }
            )
        } else if self.age < 365.0 {
            format!(
                "{} mo{}",
                (self.age / 30.0).floor(),
                if self.age / 30.0 >= 2.0 { "s" } else { "" }
            )
        } else {
            format!(
                "{} yr{}",
                (self.age / 365.0).floor(),
                if self.age / 365.0 >= 2.0 { "s" } else { "" }
            )
        }
    }

    pub fn print_weight(&self) -> String {
        if self.weight < 10.0 {
            format!("{:.1} g", self.weight)
        } else {
            format!("{} g", self.weight.round())
        }
    }

    pub fn print_fitness(&self) -> String {
        let fitness = if self.fitness.value < 0.75 {
            "(scrawny)" // TODO: "↓" not supported by tui_bit_text, find a nicer solution or just change how fitness value is shown
        } else if self.fitness.value > 1.25 {
            "(chonky)" // "↑" not supported
        } else {
            ""
        };
        fitness.to_string()
    }

    fn starting_weight() -> f32 {
        rand::random_range(RATAGOTCHI_BIRTH_WEIGHT_RANGE.0..=RATAGOTCHI_BIRTH_WEIGHT_RANGE.1)
    }

    // Key Milestone Weights
    // * Birth: 5 to 8 grams (saved in birth_weight) up to 50g
    // * Juvenile (1 to 8 months): from 50g to 200g
    // * Adult (8 months onwards): up to 450g
    fn update_weight(&mut self) {
        let milestone_months = (1.0, 8.0);
        let milestone_weights = (50.0, 200.0, 450.0);
        let months = self.age / 30.0;
        let weight = if months <= milestone_months.0 {
            self.birth_weight + months * (milestone_weights.0 - self.birth_weight)
        } else if months <= milestone_months.1 {
            let net_months = months - milestone_months.0;
            milestone_weights.0 + net_months * (milestone_weights.1 - milestone_weights.0)
        } else {
            let net_months = months - milestone_months.1;
            let exp_growth_rate = 0.5;
            let remaining_growth = milestone_weights.2 - milestone_weights.1;
            milestone_weights.2
                - (remaining_growth * (-exp_growth_rate * net_months - milestone_months.1)).exp()
        };

        self.weight = weight * self.fitness.value;
    }

    fn update_fitness(&mut self, variation: FitnessChange, multiplier: usize) {
        self.fitness.value = (self.fitness.value + variation.0 * multiplier as f32).clamp(0.5, 1.5);
        self.fitness.last_update = Utc::now();
    }

    fn fitness_decay(&mut self, time_delta: f32) {
        let num_fitness_decays = (time_delta / RATAGOTCHI_FITNESS_DECAY_INTERVAL).floor() as usize;
        self.update_fitness(
            FitnessChange(-RATAGOTCHI_FITNESS_CHANGE_S),
            num_fitness_decays,
        );
    }
}
