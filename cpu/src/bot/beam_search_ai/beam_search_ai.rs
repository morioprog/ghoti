use std::{sync::mpsc, thread, time::Instant, vec::Vec};

use puyoai::{
    color::Color,
    column_puyo_list::ColumnPuyoList,
    decision::Decision,
    es_field::EsCoreField,
    field::{self, CoreField},
    kumipuyo::{kumipuyo_seq::generate_random_puyocolor_sequence, Kumipuyo},
    plan::Plan,
    rensa_detector::{detector::detect_by_drop, PurposeForFindingRensa},
    rensa_result::RensaResult,
};

use crate::{bot::*, evaluator::Evaluator};

pub struct BeamSearchAI {
    /// 盤面の評価器
    evaluator: Evaluator,
}

impl BeamSearchAI {
    pub fn new_customize(evaluator: Evaluator) -> Self {
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
        think_frame: Option<usize>,
    ) -> AIDecision {
        let think_frame = think_frame.unwrap_or(0);
        // TODO: チューニングする
        let (depth, width) = if think_frame <= 2 {
            (20, 20)
        } else if think_frame <= 8 {
            (30, 60)
        } else {
            (40, 140)
        };
        self.think_internal(player_state_1p, player_state_2p, depth, width, 20)
    }
}

impl BeamSearchAI {
    fn think_internal(
        &self,
        player_state_1p: PlayerState,
        player_state_2p: Option<PlayerState>,
        depth: usize,
        width: usize,
        parallel: usize,
    ) -> AIDecision {
        let start = Instant::now();

        // 相手の連鎖状況を事前に計算
        let (rensa_result_2p, cf_after_chain_2p, estimated_rensa_results_2p) =
            match player_state_2p.clone() {
                Some(state) => {
                    let mut cf = state.field.clone();
                    let rensa_result_2p = cf.es_simulate_from_middle(state.current_chain);

                    let mut estimated_rensa_results_2p: Vec<RensaResult> = vec![];
                    let callback = |mut actual: CoreField, _cpl: &ColumnPuyoList| {
                        estimated_rensa_results_2p.push(actual.es_simulate());
                    };
                    detect_by_drop(
                        &cf,
                        &[false; 8],
                        PurposeForFindingRensa::ForFire,
                        3,
                        12,
                        callback,
                    );

                    (
                        Some(rensa_result_2p),
                        Some(cf),
                        Some(estimated_rensa_results_2p),
                    )
                }
                None => (None, None, None),
            };

        // NOTE: ここで渡される state は、`State::from_plan_for_fire` から返されたもの
        let third_row_height_1p = player_state_1p.field.height(3);
        let fire_condition = move |state: &State, player_state_2p: &Option<PlayerState>| -> bool {
            let plan = state.clone().plan.unwrap();
            let ojama_from_1p_chain = (plan.score() + player_state_1p.carry_over) / 70;

            // 序盤（最初の12秒）の全消し
            if player_state_1p.frame <= 60 * 12 && plan.chain() <= 3 && plan.field().is_zenkeshi() {
                return true;
            }

            // 凝視による発火判断
            if let Some(player_state_2p) = player_state_2p {
                let rensa_result_2p = rensa_result_2p.clone().unwrap();
                let cf_after_chain_2p = cf_after_chain_2p.clone().unwrap();
                let estimated_rensa_results_2p = estimated_rensa_results_2p.clone().unwrap();

                // 2Pが発火している場合
                if rensa_result_2p.score > 0 {
                    // 1P 発火のための最後のツモを引くまでのフレーム数
                    // TODO: magic number (24)
                    let frame_1p_chain_start = player_state_1p.frame + 24 + state.frame_control;
                    // 2P 連鎖終了までのフレーム数
                    let frame_2p_chain_finish = player_state_2p.frame + rensa_result_2p.frame;
                    // そもそも発火が間に合わない
                    if frame_1p_chain_start > frame_2p_chain_finish {
                        return false;
                    }
                } else {
                    // 潰し
                    // - 相手の地形が平ら
                    // - 2列以上送れそう
                    // - 3連鎖以下
                    // - いばら
                    let height_array_2p = player_state_2p.field.height_array();
                    let max_height_2p = *height_array_2p[1..7].iter().max().unwrap();
                    let min_height_2p = *height_array_2p[1..7].iter().min().unwrap();
                    let flat = max_height_2p - min_height_2p <= 1;
                    let score = plan.score() + player_state_1p.carry_over;
                    let counter = estimated_rensa_results_2p
                        .iter()
                        .filter(|res| res.chain <= 6 && res.score >= score)
                        .max_by(|a, b| a.score.cmp(&b.score));
                    if flat
                        && min_height_2p >= 2
                        && ((plan.chain() == 1 && score >= 9 * 70)
                            || (plan.chain() <= 3 && counter.is_none() && score >= 12 * 70))
                    {
                        return true;
                    }

                    // 埋まってたら条件をゆるく
                    let active_puyo = {
                        let mut cnt = 0;
                        for x in 1..=field::WIDTH {
                            let mut had_ojama = false;
                            for y in (1..=player_state_2p.field.height(x)).rev() {
                                if !player_state_2p.field.color(x, y).is_normal_color() {
                                    if had_ojama {
                                        break;
                                    }
                                    had_ojama = true;
                                } else {
                                    had_ojama = false;
                                }
                                cnt += 1;
                            }
                        }
                        cnt
                    };
                    let buried = active_puyo <= 20 && min_height_2p >= 6;
                    if buried
                        && ((min_height_2p >= 9 && score >= 3 * 70)
                            || (plan.chain() <= 3 && score >= 6 * 70))
                    {
                        return true;
                    }
                }

                // 降る予定のおじゃまぷよ（正なら自分に、負なら相手に）
                let ojama: isize = {
                    let ojama_sum_1p = player_state_1p.fixed_ojama + player_state_1p.pending_ojama;
                    let ojama_sum_2p = player_state_2p.fixed_ojama + player_state_2p.pending_ojama;
                    let ojama_from_2p_chain = if player_state_2p.current_chain > 0 {
                        (rensa_result_2p.score + player_state_2p.carry_over) / 70
                    } else {
                        0
                    };

                    (ojama_sum_1p + ojama_from_2p_chain) as isize - ojama_sum_2p as isize
                };

                // 自陣の 3 列目が埋まる可能性があるなら相殺をがんばる
                let estimated_third_row_height = third_row_height_1p
                    + ((ojama.max(0) as usize + field::WIDTH - 1) / field::WIDTH);
                if ojama > 0 && estimated_third_row_height >= 12 {
                    return ojama_from_1p_chain >= ojama as usize;
                }

                // 自分の盤面に 3 つ以上降る予定の場合
                if ojama >= 3 {
                    let ojama = ojama as usize;

                    if rensa_result_2p.chain <= 3 {
                        // 1列以上のおじゃま
                        if ojama > field::WIDTH {
                            // 1段以下に軽減する
                            return ojama_from_1p_chain + field::WIDTH >= ojama;
                        }
                        // 4列以上のおじゃま
                        if ojama >= field::WIDTH * 4 {
                            let average_height_2p: usize =
                                cf_after_chain_2p.height_array().iter().sum::<i16>() as usize
                                    / field::WIDTH;
                            // 相手の連鎖発火後に 5 段以上残ってたら副砲だと判断
                            if average_height_2p >= 5 {
                                // 使いすぎを抑制（多くても赤玉 3 個に制限する）
                                // TODO: 使いすぎないと返せない場合がありうる
                                return ojama_from_1p_chain + field::WIDTH >= ojama
                                    && ojama_from_1p_chain <= ojama + field::WIDTH * 5 * 3;
                            }
                        }
                    }

                    // TODO: 相手のセカンドを考慮する
                    return ojama_from_1p_chain >= ojama;
                }

                // 先打ち（8万点以上で考慮）
                let honsen_2p = estimated_rensa_results_2p
                    .iter()
                    .max_by(|a, b| a.score.cmp(&b.score))
                    .map_or(0, |res| res.score);
                if plan.score() < 80000 {
                    return false;
                }
                if plan.score() >= 80000 && honsen_2p + 30000 <= plan.score() {
                    return true;
                }
                if plan.score() >= 90000 && honsen_2p + 20000 <= plan.score() {
                    return true;
                }
                if plan.score() >= 100000 && honsen_2p + 10000 <= plan.score() {
                    return true;
                }
                return honsen_2p <= plan.score();
            }

            // 飽和
            return plan.score() >= 80000;
        };

        // TODO: 序盤のテンプレ化

        // 各スレッドの結果をまとめる
        let (tx, rx): (mpsc::Sender<AIDecision>, mpsc::Receiver<AIDecision>) = mpsc::channel();

        // ツモが十分に渡されてたら、モンテカルロをする必要がない
        let parallel = if player_state_1p.seq.len() < depth {
            parallel
        } else {
            1
        };

        for _ in 0..parallel {
            let depth_c = depth;
            let width_c = width;
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
    // TODO: 本線なら点数が最大のものでよいが、副砲ならそうではないはず？
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
