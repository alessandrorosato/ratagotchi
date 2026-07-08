use crate::config::{
    MINIGAME_HAPPINESS_REWARD_L, MINIGAME_HAPPINESS_REWARD_S, MINIGAME_NUM_ROUNDS,
    MINIGAME_POSTROUND_DURATION, MINIGAME_ROUND_DURATION,
};

#[derive(Debug, Default, PartialEq)]
pub struct MiniGame {
    pub round: usize,
    pub scores: [Option<bool>; 3],
    pub first_number: usize,
    pub second_number: usize,
    pub current_pick: MiniGamePick,
    pub timer: usize,
    pub phase: MiniGamePhase,
}

#[derive(Debug, Default, PartialEq)]
pub enum MiniGamePick {
    #[default]
    Higher,
    Lower,
}

#[derive(Debug, Default, PartialEq)]
pub enum MiniGamePhase {
    #[default]
    Play,
    Post,
}

impl MiniGame {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) -> Option<f32> {
        self.check_current_round()
    }

    fn tick(&mut self) {
        match self.phase {
            MiniGamePhase::Play => {
                if self.timer != 0 {
                    self.timer -= 1;
                } else {
                    self.phase = MiniGamePhase::Post;
                    self.timer = MINIGAME_POSTROUND_DURATION;
                }
            }
            MiniGamePhase::Post => {
                if self.timer != 0 {
                    self.timer -= 1;
                } else {
                    self.next_round();
                }
            }
        }
    }

    fn check_current_round(&mut self) -> Option<f32> {
        match self.round {
            0 => self.start_game(),
            1..=MINIGAME_NUM_ROUNDS => self.play_round(),
            _ => self.end_game(),
        }
    }

    fn start_game(&mut self) -> Option<f32> {
        self.round = 1;
        None
    }

    fn play_round(&mut self) -> Option<f32> {
        let round_index = self.round - 1;
        if self.scores[round_index].is_none() {
            if self.first_number == 0 {
                self.timer = MINIGAME_ROUND_DURATION;
                self.first_number = self.generate_pick();
            } else if self.timer == 2 {
                loop {
                    let second_pick = self.generate_pick();
                    if second_pick != self.first_number {
                        self.second_number = second_pick;
                        break;
                    }
                }
            } else if self.timer == 0 {
                match self.current_pick {
                    MiniGamePick::Higher => {
                        self.scores[round_index] = Some(self.first_number < self.second_number);
                    }
                    MiniGamePick::Lower => {
                        self.scores[round_index] = Some(self.first_number > self.second_number);
                    }
                }
            }
        }
        self.tick();
        None
    }

    fn next_round(&mut self) {
        self.reset_round();
        self.round += 1;
    }

    fn end_game(&mut self) -> Option<f32> {
        let mut delta_happiness: f32 = 0.0;
        for result in self.scores.into_iter().flatten() {
            delta_happiness += if result {
                MINIGAME_HAPPINESS_REWARD_L
            } else {
                MINIGAME_HAPPINESS_REWARD_S
            };
        }
        Some(delta_happiness)
    }

    fn generate_pick(&mut self) -> usize {
        let pick: usize = rand::random_range(1..10);
        pick
    }

    fn reset_round(&mut self) {
        self.first_number = 0;
        self.second_number = 0;
        self.phase = MiniGamePhase::default();
        self.current_pick = MiniGamePick::default();
    }

    pub fn print_first_number(&self) -> String {
        format!("{}", self.first_number)
    }

    pub fn print_second_number(&self) -> String {
        format!("{}", self.second_number)
    }
}
