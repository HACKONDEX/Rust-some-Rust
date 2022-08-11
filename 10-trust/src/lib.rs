#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOutcome {
    BothCooperated,
    LeftCheated,
    RightCheated,
    BothCheated,
}
#[derive(Clone, Copy)]
pub enum AgentType {
    Cheating,
    Cooperating,
    Grudger,
    Copycat,
    Detective,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tactics {
    Cheat,
    Cooperate,
}

pub trait Strategy {
    fn update_strategy(&mut self, opponent_step: Tactics);

    fn step(&self) -> Tactics;

    fn get_score(&self) -> i32;

    fn update_score(&mut self, add: i32);
}

pub struct Game {
    left_player: Box<dyn Strategy>,
    right_player: Box<dyn Strategy>,
}

impl Game {
    pub fn new(left: Box<dyn Strategy>, right: Box<dyn Strategy>) -> Self {
        Self {
            left_player: left,
            right_player: right,
        }
    }

    pub fn left_score(&self) -> i32 {
        self.left_player.get_score()
    }

    pub fn right_score(&self) -> i32 {
        self.right_player.get_score()
    }

    pub fn play_round(&mut self) -> RoundOutcome {
        let l = self.left_player.step();
        let r = self.right_player.step();
        self.left_player.update_strategy(r);
        self.right_player.update_strategy(l);

        let (mut left_add, mut right_add) = (0, 0);
        let result = match (l, r) {
            (Tactics::Cheat, Tactics::Cheat) => RoundOutcome::BothCheated,
            (Tactics::Cheat, Tactics::Cooperate) => {
                left_add = 3;
                right_add = -1;
                RoundOutcome::LeftCheated
            }
            (Tactics::Cooperate, Tactics::Cheat) => {
                left_add = -1;
                right_add = 3;
                RoundOutcome::RightCheated
            }
            (Tactics::Cooperate, Tactics::Cooperate) => {
                left_add = 2;
                right_add = 2;
                RoundOutcome::BothCooperated
            }
        };

        self.left_player.update_score(left_add);
        self.right_player.update_score(right_add);
        result
    }
}

pub struct Player {
    score: i32,
    next_step: Tactics,
}

impl Player {
    fn new(start_tactics: Tactics) -> Self {
        Player {
            score: 0,
            next_step: start_tactics,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CheatingAgent {
    player: Player,
}

impl Strategy for CheatingAgent {
    fn update_strategy(&mut self, _opponent_step: Tactics) {}

    fn step(&self) -> Tactics {
        self.player.next_step
    }

    fn get_score(&self) -> i32 {
        self.player.score
    }

    fn update_score(&mut self, add: i32) {
        self.player.score += add;
    }
}

impl Default for CheatingAgent {
    fn default() -> Self {
        CheatingAgent {
            player: Player::new(Tactics::Cheat),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CooperatingAgent {
    player: Player,
}

impl Default for CooperatingAgent {
    fn default() -> Self {
        CooperatingAgent {
            player: Player::new(Tactics::Cooperate),
        }
    }
}

impl Strategy for CooperatingAgent {
    fn update_strategy(&mut self, _opponent_step: Tactics) {}

    fn step(&self) -> Tactics {
        self.player.next_step
    }

    fn get_score(&self) -> i32 {
        self.player.score
    }

    fn update_score(&mut self, add: i32) {
        self.player.score += add;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct GrudgerAgent {
    player: Player,
    cheated_back: bool,
}

impl Default for GrudgerAgent {
    fn default() -> Self {
        GrudgerAgent {
            player: Player::new(Tactics::Cooperate),
            cheated_back: false,
        }
    }
}

impl Strategy for GrudgerAgent {
    fn update_strategy(&mut self, opponent_step: Tactics) {
        if !self.cheated_back && opponent_step == Tactics::Cheat {
            self.cheated_back = true;
            self.player.next_step = Tactics::Cheat;
        }
    }

    fn step(&self) -> Tactics {
        self.player.next_step
    }

    fn get_score(&self) -> i32 {
        self.player.score
    }

    fn update_score(&mut self, add: i32) {
        self.player.score += add;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct CopycatAgent {
    player: Player,
}

impl Default for CopycatAgent {
    fn default() -> Self {
        Self {
            player: Player::new(Tactics::Cooperate),
        }
    }
}

impl Strategy for CopycatAgent {
    fn update_strategy(&mut self, opponent_step: Tactics) {
        self.player.next_step = opponent_step
    }

    fn step(&self) -> Tactics {
        self.player.next_step
    }

    fn get_score(&self) -> i32 {
        self.player.score
    }

    fn update_score(&mut self, add: i32) {
        self.player.score += add;
    }
}

////////////////////////////////////////////////////////////////////////////////

// use std::result;

pub struct DetectiveAgent {
    player: Player,
    rounde: i32,
    cheated_back: bool,
    copycat: CopycatAgent,
}

impl Default for DetectiveAgent {
    fn default() -> Self {
        Self {
            player: Player::new(Tactics::Cooperate),
            rounde: 0,
            cheated_back: false,
            copycat: CopycatAgent::default(),
        }
    }
}

impl Strategy for DetectiveAgent {
    fn update_strategy(&mut self, opponent_step: Tactics) {
        if self.rounde <= 2 {
            match self.rounde {
                0 => self.player.next_step = Tactics::Cheat,
                1 => self.player.next_step = Tactics::Cooperate,
                2 => self.player.next_step = Tactics::Cooperate,
                _ => {}
            }

            self.rounde += 1;

            if !self.cheated_back && opponent_step == Tactics::Cheat {
                self.cheated_back = true;
            }
        } else if self.cheated_back {
            self.copycat.update_strategy(opponent_step);
            self.player.next_step = self.copycat.player.next_step;
        } else {
            self.player.next_step = Tactics::Cheat;
        }
    }

    fn step(&self) -> Tactics {
        self.player.next_step
    }

    fn get_score(&self) -> i32 {
        self.player.score
    }

    fn update_score(&mut self, add: i32) {
        self.player.score += add;
    }
}
