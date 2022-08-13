use std::{time::Duration, vec::Vec};

use puyoai::{decision::Decision, field::core_field::CoreField, kumipuyo::kumipuyo::Kumipuyo};

pub trait AI {
    fn new() -> Self
    where
        Self: Sized;
    fn name(&self) -> &'static str;
    fn think(
        &self,
        player_state_1p: PlayerState,
        player_state_2p: Option<PlayerState>,
    ) -> AIDecision;
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
    /// 点数
    pub score: usize,
    /// 点数の端数・落下ボーナス・全消しボーナス の総和
    pub carry_over: usize,
    /// 確定おじゃまぷよ
    pub fixed_ojama: usize,
    /// 予告おじゃまぷよ
    pub pending_ojama: usize,
    /// 予告おじゃまぷよがいつ確定するか
    pub ojama_committing_frame_id: usize,
    /// ツモ番号
    pub tumo_index: usize,
    /// 配ぷよ全体
    haipuyo: Option<Vec<Kumipuyo>>,
}

impl PlayerState {
    pub fn new(
        frame: usize,
        field: CoreField,
        seq: Vec<Kumipuyo>,
        score: usize,
        carry_over: usize,
        fixed_ojama: usize,
        pending_ojama: usize,
        ojama_committing_frame_id: usize,
        tumo_index: usize,
        haipuyo: Option<Vec<Kumipuyo>>,
    ) -> Self {
        PlayerState {
            frame,
            field,
            seq,
            score,
            carry_over,
            fixed_ojama,
            pending_ojama,
            ojama_committing_frame_id,
            tumo_index,
            haipuyo,
        }
    }
    pub fn initial_state(seq: Vec<Kumipuyo>, haipuyo: Option<Vec<Kumipuyo>>) -> Self {
        PlayerState {
            frame: 0,
            field: CoreField::new(),
            seq,
            score: 0,
            carry_over: 0,
            fixed_ojama: 0,
            pending_ojama: 0,
            ojama_committing_frame_id: 0,
            tumo_index: 0,
            haipuyo,
        }
    }
    pub fn zero() -> Self {
        PlayerState {
            frame: 0,
            field: CoreField::new(),
            seq: vec![],
            score: 0,
            carry_over: 0,
            fixed_ojama: 0,
            pending_ojama: 0,
            ojama_committing_frame_id: 0,
            tumo_index: 0,
            haipuyo: None,
        }
    }

    pub fn set_seq(&mut self, visible_tumos: usize) {
        debug_assert!(self.haipuyo.is_some());
        if let Some(haipuyo) = &self.haipuyo {
            let seq = {
                let mut seq = vec![];
                for i in 0..visible_tumos {
                    seq.push(haipuyo[(self.tumo_index + i) % haipuyo.len()].clone());
                }
                seq
            };
            self.seq = seq;
        };
    }
    pub fn has_haipuyo(&self) -> bool {
        self.haipuyo.is_some()
    }
    pub fn set_haipuyo(&mut self, haipuyo: Vec<Kumipuyo>) {
        debug_assert!(!self.has_haipuyo());
        self.haipuyo = Some(haipuyo);
    }
    pub fn drop_kumipuyo(&mut self, decision: &Decision) {
        self.field.drop_kumipuyo(decision, &self.seq[0]);
    }
}
