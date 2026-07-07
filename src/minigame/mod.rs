use crate::config::{MINIGAME_NUM_ROUNDS, MINIGAME_ROUND_DURATION};

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

    pub fn run(&mut self) {
        self.check_current_round();
    }

    fn tick(&mut self) {
        match self.phase {
            MiniGamePhase::Play => {
                if self.timer != 0 {
                    self.timer -= 1;
                } else {
                    self.phase = MiniGamePhase::Post;
                }
            }
            MiniGamePhase::Post => todo!(),
        }
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
        self.tick();
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
