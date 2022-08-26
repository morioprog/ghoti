use std::{sync::mpsc, thread, time::Instant, vec::Vec};

use puyoai::{
    decision::Decision,
    field::CoreField,
    kumipuyo::{kumipuyo_seq::generate_random_puyocolor_sequence, Kumipuyo},
    plan::Plan,
};

use crate::{bot::*, evaluator::Evaluator};

pub struct BeamSearchAI {
    /// 盤面の評価器
    evaluator: Evaluator,
    /// ビームサーチの深さ
    depth: usize,
    /// ビーム幅
    width: usize,
    /// モンテカルロを何並列でするか
    parallel: usize,
}

impl BeamSearchAI {
    pub fn new_customize(
        evaluator: Evaluator,
        depth: usize,
        width: usize,
        parallel: usize,
    ) -> Self {
        BeamSearchAI {
            evaluator,
            depth,
            width,
            parallel,
        }
    }
}

impl AI for BeamSearchAI {
    fn new() -> Self {
        BeamSearchAI {
            evaluator: Evaluator::default(),
            depth: 30,
            width: 100,
            parallel: 20,
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

        // NOTE: ここで渡される state は、`State::from_plan_for_fire` から返されたもの
        let fire_condition = move |state: &State, _player_state_2p: &Option<PlayerState>| -> bool {
            let plan = state.clone().plan.unwrap();

            // 1P 発火のための最後のツモを引くまでのフレーム数
            // TODO: magic number (24)
            let _frame_1p_chain_start = player_state_1p.frame + 24 + state.frame_control;

            // TODO: 凝視による発火判断
            return plan.score() >= 70000 || (plan.chain() <= 3 && plan.field().is_zenkeshi());
        };

        // TODO: 序盤のテンプレ化

        // 各スレッドの結果をまとめる
        let (tx, rx): (mpsc::Sender<AIDecision>, mpsc::Receiver<AIDecision>) = mpsc::channel();

        // ツモが十分に渡されてたら、モンテカルロをする必要がない
        let parallel = if player_state_1p.seq.len() < self.depth {
            self.parallel
        } else {
            1
        };

        for _ in 0..parallel {
            let depth_c = self.depth;
            let width_c = self.width;
            let tx_c = tx.clone();
            let player_state_1p_c = player_state_1p.clone();
            let player_state_2p_c = player_state_2p.clone();
            let fire_condition_c = fire_condition.clone();
            let evaluator_c = self.evaluator.clone();

            thread::spawn(move || {
                tx_c.send(think_single_thread(
                    depth_c,
                    width_c,
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

        let best_decision = Decision::all_valid_decisions()
            .iter()
            .max_by(|d1, d2| scores[d1.axis_x()][d1.rot()].cmp(&scores[d2.axis_x()][d2.rot()]))
            .unwrap();

        if let Some(ai_decision) = ai_decisions
            .iter()
            .find(|&ai_decision| &ai_decision.decisions[0] == best_decision)
        {
            return AIDecision::new(
                ai_decision.decisions.clone(),
                ai_decision.log_output.clone(),
                start.elapsed(),
            );
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
    frame_control: usize,
    /// そのツモを置くまで or 連鎖が終わり、相手にお邪魔が降るまでのフレーム数
    frame_chain: usize,
}

impl State {
    fn empty() -> Self {
        State {
            field: CoreField::new(),
            decisions: vec![],
            eval_score: 0_i32,
            frame_control: 0_usize,
            frame_chain: 0_usize,
            plan: None,
        }
    }

    fn from_plan(
        plan: &Plan,
        decisions: Vec<Decision>,
        eval_score: i32,
        // それまでの操作に必要なフレーム数の総和
        frame_margin: usize,
    ) -> Self {
        State {
            field: plan.field().clone(),
            decisions,
            eval_score: eval_score,
            frame_control: frame_margin + plan.frames_to_ignite() + plan.last_drop_frames(),
            frame_chain: frame_margin + plan.total_frames(),
            plan: Some(plan.clone()),
        }
    }

    /// 発火時のState（多分別のstructに分けた方がいいけど、面倒なのでこのままで）
    fn from_plan_for_fire(
        plan: &Plan,
        decisions: Vec<Decision>,
        // 発火した連鎖の点数
        eval_score: i32,
        // 発火時のツモ以外の操作に必要なフレーム数の総和（本来の定義とは異なることに注意）
        frame_control: usize,
    ) -> Self {
        State {
            field: plan.field().clone(),
            decisions,
            eval_score,
            frame_control,
            frame_chain: frame_control + plan.total_frames(),
            plan: Some(plan.clone()),
        }
    }

    // NOTE: 初期化にしか使っていない
    fn from_field(field: &CoreField) -> Self {
        State {
            field: field.clone(),
            decisions: vec![],
            eval_score: 0_i32,
            frame_control: 0_usize,
            frame_chain: 0_usize,
            plan: None,
        }
    }

    fn first_decision(&self) -> Option<&Decision> {
        self.decisions.first()
    }
}

fn generate_next_states(
    cur_state: &State,
    next_states: &mut Vec<State>,
    fired: &mut Vec<State>,
    kumipuyo: &Kumipuyo,
    append_fired: bool,
    evaluator: &Evaluator,
) {
    let decisions = &cur_state.decisions;
    let seq = vec![kumipuyo.clone()];

    Plan::iterate_available_plans(&cur_state.field, &seq, 1, &mut |plan: &Plan| {
        // TODO: どうにかできそう
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
                cur_state.frame_control,
            ))
        }

        next_states.push(State::from_plan(
            plan,
            ds.clone(),
            evaluator.evaluate(plan),
            cur_state.frame_control,
        ));
    });
}

fn think_single_thread<F>(
    depth: usize,
    width: usize,
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
    let seq: Vec<Kumipuyo> = seq
        .iter()
        .cloned()
        .chain(generate_random_puyocolor_sequence(
            if depth > visible_tumos {
                depth - visible_tumos
            } else {
                0
            },
        ))
        .collect();

    let mut state_v: Vec<State> = vec![State::from_field(cf)];
    let mut fired_v: Vec<State> =
        Vec::with_capacity(width * Decision::all_valid_decisions().len() * depth);

    for depth in 0..depth {
        // ビーム内の初手がすべて同じなら終わり
        if depth > 0
            && state_v
                .iter()
                .all(|state| state.first_decision() == state_v[0].first_decision())
        {
            break;
        }

        // 次の状態を列挙
        let mut next_state_v: Vec<State> =
            Vec::with_capacity(width * Decision::all_valid_decisions().len());
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
        if next_state_v.len() > width {
            next_state_v.resize(width, State::empty());
        }
        state_v = next_state_v;
    }

    // 発火すべきものがあれば、その中で一番点数が高いものを選んでいる
    if let Some(fire) = fired_v
        .iter()
        .filter(|f| fire_condition(f, player_state_2p))
        .max_by(|f1, f2| f1.eval_score.cmp(&f2.eval_score))
    {
        return AIDecision::new(
            fire.decisions.clone(),
            format!(
                "fire: {:6}\n{:4} F -> {:4} F",
                fire.eval_score, fire.frame_control, fire.frame_chain,
            ),
            start.elapsed(),
        );
    }

    if state_v[0].first_decision().is_some() {
        return AIDecision::new(
            state_v[0].decisions.clone(),
            format!("eval: {:6}", state_v[0].eval_score),
            start.elapsed(),
        );
    }

    // どうしようもないので自殺
    return AIDecision::new(
        vec![Decision::new(3, 0)],
        format!("muri..."),
        start.elapsed(),
    );
}
