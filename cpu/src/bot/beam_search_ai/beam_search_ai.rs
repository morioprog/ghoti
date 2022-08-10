use std::{sync::mpsc, thread, time::Instant, vec::Vec};

use puyoai::{
    decision::Decision,
    field::CoreField,
    kumipuyo::{kumipuyo_seq::generate_random_puyocolor_sequence, Kumipuyo},
    plan::Plan,
};

use crate::{bot::*, evaluator::Evaluator};

/// ビームサーチの深さ
const DEPTH: usize = 30;
/// ビーム幅
const WIDTH: usize = 100;
/// モンテカルロを何並列でするか
const PARALLEL: usize = 20;

fn generate_next_states(
    cur_state: &State,
    next_states: &mut Vec<State>,
    fired: &mut Vec<State>,
    kumipuyo: &Kumipuyo,
    append_fired: bool,
    evaluator: &Evaluator,
) {
    let decisions = cur_state.decisions.clone();
    let seq = vec![kumipuyo.clone()];

    Plan::iterate_available_plans(&cur_state.field, &seq, 1, &mut |plan: &Plan| {
        let ds = {
            let mut ds = decisions.clone();
            ds.append(&mut vec![plan.first_decision().clone()]);
            ds
        };

        if append_fired && plan.chain() > 0 {
            fired.push(State::from_plan_for_fire(
                plan,
                ds.clone(),
                plan.score() as i32,
                cur_state.frame_controll,
            ))
        }

        next_states.push(State::from_plan(
            plan,
            ds.clone(),
            evaluator.evaluate(plan),
            cur_state.frame_controll,
        ));
    });
}

fn think_single_thread<F>(
    player_state_1p: &PlayerState,
    player_state_2p: &Option<PlayerState>,
    fire_condition: F,
    evaluator: &Evaluator,
) -> AIDecision
where
    F: Fn(&State, &Option<PlayerState>) -> bool,
{
    let start = Instant::now();

    let cf = &player_state_1p.field;
    let seq = &player_state_1p.seq;

    // ツモを伸ばす（モンテカルロ）
    let visible_tumos = seq.len();
    let seq = {
        let mut seq = seq.clone();
        if visible_tumos < DEPTH {
            seq.append(&mut generate_random_puyocolor_sequence(
                DEPTH - visible_tumos,
            ));
        }
        seq
    };

    let mut state_v: Vec<State> = vec![State::from_field(cf)];
    let mut fired_v: Vec<State> = Vec::with_capacity(WIDTH * 22 * DEPTH);

    for depth in 0..DEPTH {
        // ビーム内の初手がすべて同じなら終わり
        if depth > 0 && {
            let mut fin = true;
            for i in 1..state_v.len() {
                if state_v[0].first_decision() != state_v[i].first_decision() {
                    fin = false;
                    break;
                }
            }
            fin
        } {
            break;
        }

        // 次の状態を列挙
        let mut next_state_v: Vec<State> = Vec::with_capacity(WIDTH * 22);
        for cur_state in &state_v {
            generate_next_states(
                &cur_state,
                &mut next_state_v,
                &mut fired_v,
                &seq[depth],
                depth < visible_tumos,
                evaluator,
            );
        }
        if next_state_v.is_empty() {
            break;
        }

        // 良い方からビーム幅分だけ残す
        next_state_v
            .sort_by(|a: &State, b: &State| (-a.eval_score).partial_cmp(&-b.eval_score).unwrap());
        if next_state_v.len() > WIDTH {
            next_state_v.resize(WIDTH, State::empty());
        }
        state_v = next_state_v;
    }

    if !fired_v.is_empty() {
        // NOTE: conditionを満たすものの中で、一番点数が高いものを選んでいる
        let highest_idx = {
            let mut idx = 0;
            for i in 1..fired_v.len() {
                if fire_condition(&fired_v[i], player_state_2p)
                    && fired_v[i].eval_score > fired_v[idx].eval_score
                {
                    idx = i;
                }
            }
            idx
        };

        // 発火した連鎖ならなんでも突っ込んでるので、この条件が必要
        if fire_condition(&fired_v[highest_idx], player_state_2p) {
            return AIDecision::new(
                fired_v[highest_idx].decisions.clone(),
                format!(
                    "> fire score\n {:5}\n ({}, {})\n {:4} F -> {:4} F",
                    fired_v[highest_idx].eval_score,
                    fired_v[highest_idx].first_decision().unwrap().axis_x(),
                    fired_v[highest_idx].first_decision().unwrap().rot(),
                    fired_v[highest_idx].frame_controll,
                    fired_v[highest_idx].frame_chain,
                ),
                start.elapsed(),
            );
        }
    }

    // どうしようもないので自殺
    if state_v[0].first_decision().is_none() {
        // TODO: 特に `state_v[0]` を再利用する必要はない
        state_v[0].decisions.push(Decision::new(3, 0));
        return AIDecision::new(
            state_v[0].decisions.clone(),
            format!("> muri...\n\n (3, 0)"),
            start.elapsed(),
        );
    }

    AIDecision::new(
        state_v[0].decisions.clone(),
        format!(
            "> eval score\n {:5}\n ({}, {})",
            state_v[0].eval_score,
            state_v[0].first_decision().unwrap().axis_x(),
            state_v[0].first_decision().unwrap().rot(),
        ),
        start.elapsed(),
    )
}

pub struct BeamSearchAI {
    evaluator: Evaluator,
}

impl BeamSearchAI {
    pub fn with_evaluator(evaluator: Evaluator) -> Self {
        BeamSearchAI { evaluator }
    }
}

impl AI for BeamSearchAI {
    fn new() -> Self {
        BeamSearchAI {
            evaluator: Evaluator::default(),
        }
    }

    fn name(&self) -> &'static str {
        "BeamSearchAI"
    }

    fn think(
        &self,
        player_state_1p: PlayerState,
        player_state_2p: Option<PlayerState>,
    ) -> AIDecision {
        let start = Instant::now();

        // NOTE: このstateは、`State::from_plan_for_fire` で返されたもの
        let fire_condition = move |state: &State, _player_state_2p: &Option<PlayerState>| -> bool {
            let plan = state.clone().plan.unwrap();

            // 1P 発火のための最後のツモを引くまでのフレーム数
            // TODO: magic number (24)
            let _frame_1p_chain_start = player_state_1p.frame + 24 + state.frame_controll;

            // TODO: 凝視による発火判断
            return plan.score() >= 70000 || (plan.chain() <= 3 && plan.field().is_zenkeshi());
        };

        // TODO: 序盤のテンプレ化

        // 各スレッドの結果をまとめる
        let (tx, rx): (mpsc::Sender<AIDecision>, mpsc::Receiver<AIDecision>) = mpsc::channel();

        // ツモが十分に渡されてたら、モンテカルロをする必要がない
        let parallel = if player_state_1p.seq.len() < DEPTH {
            PARALLEL
        } else {
            1
        };

        for _ in 0..parallel {
            let tx_c = tx.clone();
            let player_state_1p_c = player_state_1p.clone();
            let player_state_2p_c = player_state_2p.clone();
            let fire_condition_c = fire_condition.clone();
            let evaluator_c = self.evaluator.clone();

            thread::spawn(move || {
                tx_c.send(think_single_thread(
                    &player_state_1p_c,
                    &player_state_2p_c,
                    fire_condition_c,
                    &evaluator_c,
                ))
                .ok();
            });
        }

        // scores[x][r] := 解として選ばれた回数
        let mut scores = [[0_i32; 4]; 7];
        let mut ai_decisions = Vec::with_capacity(parallel);

        for _ in 0..parallel {
            if let Ok(ai_decision) = rx.recv() {
                // 打てるものがあったらすぐにそれを打つ（全部のスレッドでそういう結果なはずなので）
                // TODO: 判定方法が ad-hoc すぎるのでもうちょっといい方法を考える
                if ai_decision.log_output.contains("fire") {
                    return AIDecision::new(
                        ai_decision.decisions.clone(),
                        ai_decision.log_output.clone(),
                        start.elapsed(),
                    );
                }

                let first_decision = &ai_decision.decisions[0];
                let x = first_decision.axis_x();
                let r = first_decision.rot();
                scores[x][r] += 1;
                ai_decisions.push(ai_decision);
            } else {
                break;
            }
        }

        let mut best_decision = Decision::new(3, 0);
        for decision in Decision::all_valid_decisions() {
            let x = decision.axis_x();
            let r = decision.rot();

            let bx = best_decision.axis_x();
            let br = best_decision.rot();

            if scores[x][r] > scores[bx][br] {
                best_decision = decision.clone();
            }
        }

        for ai_decision in ai_decisions {
            if ai_decision.decisions[0] == best_decision {
                return AIDecision::new(
                    ai_decision.decisions.clone(),
                    ai_decision.log_output.clone(),
                    start.elapsed(),
                );
            }
        }

        // 死ぬしかない状態でも "muri..." が入っているはずなので
        unreachable!()
    }
}

#[derive(Clone)]
struct State {
    field: CoreField,
    decisions: Vec<Decision>,
    eval_score: i32,
    plan: Option<Plan>,
    /// 発火時のツモを置くまでに必要なフレーム数
    frame_controll: usize,
    /// そのツモを置くまで or 連鎖が終わり、相手にお邪魔が降るまでのフレーム数
    frame_chain: usize,
}

impl State {
    fn empty() -> Self {
        State {
            field: CoreField::new(),
            decisions: vec![],
            eval_score: 0_i32,
            frame_controll: 0_usize,
            frame_chain: 0_usize,
            plan: None,
        }
    }

    /// `frame_margin`: それまでの操作に必要なフレーム数の総和
    fn from_plan(
        plan: &Plan,
        decisions: Vec<Decision>,
        eval_score: i32,
        frame_margin: usize,
    ) -> Self {
        State {
            field: plan.field().clone(),
            decisions,
            eval_score: eval_score,
            frame_controll: frame_margin + plan.frames_to_ignite() + plan.last_drop_frames(),
            frame_chain: frame_margin + plan.total_frames(),
            plan: Some(plan.clone()),
        }
    }

    /// 発火時のState（多分別のstructに分けた方がいいけど、面倒なのでこのままで）
    /// - `eval_score`: 発火した連鎖の点数
    /// - `frame_controll`: 発火時のツモ以外の操作に必要なフレーム数の総和（本来の定義とは異なることに注意）
    fn from_plan_for_fire(
        plan: &Plan,
        decisions: Vec<Decision>,
        eval_score: i32,
        frame_controll: usize,
    ) -> Self {
        State {
            field: plan.field().clone(),
            decisions,
            eval_score,
            frame_controll,
            frame_chain: frame_controll + plan.total_frames(),
            plan: Some(plan.clone()),
        }
    }

    // 初期化にしか使っていない
    fn from_field(field: &CoreField) -> Self {
        State {
            field: field.clone(),
            decisions: vec![],
            eval_score: 0_i32,
            frame_controll: 0_usize,
            frame_chain: 0_usize,
            plan: None,
        }
    }

    fn first_decision(&self) -> Option<&Decision> {
        self.decisions.first()
    }
}
