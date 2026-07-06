use crate::config::{MINIGAME_NUM_ROUNDS, MINIGAME_ROUND_DURATION, RATAGOTCHI_ACTIVITY_DURATION};

#[derive(Clone, Debug, PartialEq)]
pub struct MiniGame {
    pub round: usize,
    pub scores: [Option<bool>; 3],
    pub first_number: usize,
    pub second_number: usize,
    pub current_pick: MiniGamePick,
    pub timer: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum MiniGamePick {
    #[default]
    Higher,
    Lower,
}

impl Default for MiniGame {
    fn default() -> Self {
        MiniGame {
            round: 0,
            scores: [None, None, None],
            first_number: 0,
            second_number: 0,
            current_pick: MiniGamePick::default(),
            timer: RATAGOTCHI_ACTIVITY_DURATION,
        }
    }
}

impl MiniGame {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) {
        self.check_current_round();
    }

    fn check_current_round(&mut self) {
        match self.round {
            0 => self.start_game(),
            1..=MINIGAME_NUM_ROUNDS => self.play_round(),
            _ => self.end_game(),
        }
    }

    fn start_game(&mut self) {
        self.round = 1;
    }

    fn play_round(&mut self) {
        if self.scores[self.round].is_none() {
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
                        self.scores[self.round] = Some(self.first_number < self.second_number);
                    }
                    MiniGamePick::Lower => {
                        self.scores[self.round] = Some(self.first_number > self.second_number);
                    }
                }
            }
        } else {
            // Move to next round
            self.next_round();
        }
    }

    fn next_round(&mut self) {
        self.round += 1;
    }

    fn end_game(&mut self) {}

    fn generate_pick(&mut self) -> usize {
        let pick: usize = rand::random_range(1..10);
        pick
    }
}
