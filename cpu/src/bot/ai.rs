use std::{time::Duration, vec::Vec};

use puyoai::{decision::Decision, field::core_field::CoreField, kumipuyo::kumipuyo::Kumipuyo};

pub trait AI {
    fn new() -> Self
    where
        Self: Sized;
    fn name(&self) -> &'static str;
    fn think(&self, player_state_1p: &PlayerState, player_state_2p: &PlayerState) -> AIDecision;
}

#[derive(Clone)]
pub struct AIDecision {
    pub decisions: Vec<Decision>,
    pub log_output: String,
    pub elapsed: Duration,
}

impl AIDecision {
    pub fn new(decisions: Vec<Decision>, log_output: String, elapsed: Duration) -> Self {
        AIDecision {
            decisions,
            log_output,
            elapsed,
        }
    }
    pub fn from_decision(decision: &Decision, log_output: String, elapsed: Duration) -> Self {
        AIDecision {
            decisions: vec![decision.clone()],
            log_output,
            elapsed,
        }
    }
    pub fn zero() -> Self {
        AIDecision {
            decisions: vec![Decision::new(0, 0)],
            log_output: "".to_string(),
            elapsed: Duration::ZERO,
        }
    }
}

#[derive(Clone)]
pub struct PlayerState {
    /// 試合開始からのフレーム数
    pub frame: usize,
    /// 上記フレーム数時点での盤面
    pub field: CoreField,
    /// ツモ
    pub seq: Vec<Kumipuyo>,
    /// 点数の端数・落下ボーナス・全消しボーナス の総和
    pub carry_over: usize,
    /// 確定おじゃまぷよ
    pub fixed_ojama: usize,
    /// 予告おじゃまぷよ
    pub pending_ojama: usize,
    /// 予告おじゃまぷよがいつ確定するか
    pub ojama_committing_frame_id: usize,
}

impl PlayerState {
    pub fn new(
        frame: usize,
        field: CoreField,
        seq: Vec<Kumipuyo>,
        carry_over: usize,
        fixed_ojama: usize,
        pending_ojama: usize,
        ojama_committing_frame_id: usize,
    ) -> Self {
        PlayerState {
            frame,
            field,
            seq,
            carry_over,
            fixed_ojama,
            pending_ojama,
            ojama_committing_frame_id,
        }
    }
    pub fn zero() -> Self {
        PlayerState {
            frame: 0,
            field: CoreField::new(),
            seq: vec![],
            carry_over: 0,
            fixed_ojama: 0,
            pending_ojama: 0,
            ojama_committing_frame_id: 0,
        }
    }
}
