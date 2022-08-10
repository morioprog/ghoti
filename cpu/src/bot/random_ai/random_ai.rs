use std::time::Instant;

use puyoai::{control::PuyoController, decision::Decision};
use rand::seq::SliceRandom;

use crate::bot::*;

pub struct RandomAI {}

impl AI for RandomAI {
    fn new() -> Self {
        RandomAI {}
    }

    fn name(&self) -> &'static str {
        "RandomAI"
    }

    fn think(&self, player_state_1p: &PlayerState, _player_state_2p: &PlayerState) -> AIDecision {
        let start = Instant::now();
        let controller = PuyoController::new();

        let mut retry = 0;
        loop {
            let decision = Decision::all_valid_decisions()
                .choose(&mut rand::thread_rng())
                .unwrap();
            if controller.is_reachable(&player_state_1p.field, decision) {
                return AIDecision::from_decision(
                    decision,
                    format!("Random (retry: {retry})"),
                    start.elapsed(),
                );
            }
            retry += 1;
        }
    }
}
