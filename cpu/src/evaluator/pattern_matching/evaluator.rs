use puyoai::{
    color::Color,
    column_puyo_list::ColumnPuyoList,
    field::{self, CoreField},
    plan::Plan,
    rensa_detector::{detector::detect_by_drop, PurposeForFindingRensa},
    rensa_result::RensaResult,
};
use serde::{Deserialize, Serialize};

use super::detect_shape::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evaluator {
    // 盤面
    pub valley: i32,
    pub ridge: i32,
    pub ideal_height_diff: i32,
    pub ideal_height_diff_sq: i32,
    pub ideal_height_coef_1: i32,
    pub ideal_height_coef_2: i32,
    pub ideal_height_coef_3: i32,
    pub ideal_height_coef_4: i32,
    pub third_column_height: i32,
    pub third_column_height_sq: i32,
    pub unreachable_space: i32,
    pub top_row: [i32; field::WIDTH],
    // 連結
    pub connectivity_2: i32,
    pub connectivity_3: i32,
    // 発生した連鎖
    pub chain: i32,
    pub chain_sq: i32,
    pub chain_score: i32,
    pub chain_frame: i32,
    // 盤面から起こりうる連鎖
    pub potential_main_chain: i32,
    pub potential_main_chain_sq: i32,
    pub potential_main_chain_frame: i32,
    pub potential_main_chain_ignition_height: i32,
    pub potential_sub_chain: i32,
    pub potential_sub_chain_sq: i32,
    pub potential_sub_chain_frame: i32,
    pub potential_sub_chain_ignition_height: i32,
    // フレーム関係
    pub chigiri: i32,
    pub move_frame: i32,
    // パターンマッチング
    pub gtr_base_1: i32,
    pub gtr_base_2: i32,
    pub gtr_base_3: i32,
    pub gtr_base_4: i32,
    pub gtr_base_5: i32,
    pub gtr_base_6: i32,
    pub gtr_base_7: i32,
    pub gtr_1: i32,
    pub gtr_2: i32,
    pub gtr_3: i32,
    pub gtr_4: i32,
    pub gtr_5: i32,
    pub gtr_6: i32,
    pub gtr_tail_1_1: i32,
    pub gtr_tail_1_2: i32,
    pub gtr_tail_1_3: i32,
    pub gtr_tail_2_1: i32,
    pub gtr_tail_2_2: i32,
    pub gtr_tail_2_3: i32,
    pub gtr_tail_2_4: i32,
    pub gtr_tail_2_5: i32,
    pub gtr_tail_2_6: i32,
    pub gtr_tail_2_7: i32,
    pub gtr_tail_3_1: i32,
    pub gtr_tail_3_2: i32,
    pub gtr_tail_3_3: i32,
    pub gtr_tail_3_4: i32,
    pub gtr_tail_4_1: i32,
    pub gtr_tail_5_1: i32,
    pub gtr_tail_5_2: i32,
    pub gtr_tail_6_1: i32,
    pub gtr_tail_6_2: i32,
    pub gtr_tail_6_3: i32,
    pub gtr_head_1: i32,
    pub gtr_head_2: i32,
    pub gtr_head_3: i32,
    pub gtr_head_4: i32,
    pub gtr_head_5: i32,
    pub gtr_head_6: i32,
    // チューニング用
    pub sub_name: Option<String>,
}

impl Default for Evaluator {
    fn default() -> Self {
        return Self {
            // 盤面
            valley: -352,
            ridge: -84,
            ideal_height_diff: 307,
            ideal_height_diff_sq: -681,
            ideal_height_coef_1: 124,
            ideal_height_coef_2: 590,
            ideal_height_coef_3: 310,
            ideal_height_coef_4: 754,
            third_column_height: 356,
            third_column_height_sq: -19,
            unreachable_space: -339,
            top_row: [-21, -237, 154, 391, 506, -74],
            // 連結
            connectivity_2: 52,
            connectivity_3: 345,
            // 発生した連鎖
            chain: 201,
            chain_sq: -96,
            chain_score: 538,
            chain_frame: 18,
            // 盤面から起こりうる連鎖
            potential_main_chain: 311,
            potential_main_chain_sq: 145,
            potential_main_chain_frame: 99,
            potential_main_chain_ignition_height: 658,
            potential_sub_chain: 350,
            potential_sub_chain_sq: -154,
            potential_sub_chain_frame: -22,
            potential_sub_chain_ignition_height: 466,
            // フレーム関係
            chigiri: -29,
            move_frame: -559,
            // TODO: パターンマッチ用
            gtr_base_1: 20,
            gtr_base_2: 20,
            gtr_base_3: 20,
            gtr_base_4: 20,
            gtr_base_5: 20,
            gtr_base_6: 20,
            gtr_base_7: 20,
            gtr_1: 50,
            gtr_2: 50,
            gtr_3: 50,
            gtr_4: 50,
            gtr_5: 50,
            gtr_6: 50,
            gtr_tail_1_1: 30,
            gtr_tail_1_2: 30,
            gtr_tail_1_3: 30,
            gtr_tail_2_1: 30,
            gtr_tail_2_2: 30,
            gtr_tail_2_3: 30,
            gtr_tail_2_4: 30,
            gtr_tail_2_5: 30,
            gtr_tail_2_6: 30,
            gtr_tail_2_7: 30,
            gtr_tail_3_1: 30,
            gtr_tail_3_2: 30,
            gtr_tail_3_3: 30,
            gtr_tail_3_4: 30,
            gtr_tail_4_1: 30,
            gtr_tail_5_1: 30,
            gtr_tail_5_2: 30,
            gtr_tail_6_1: 30,
            gtr_tail_6_2: 30,
            gtr_tail_6_3: 30,
            gtr_head_1: 30,
            gtr_head_2: 30,
            gtr_head_3: 30,
            gtr_head_4: 30,
            gtr_head_5: 30,
            gtr_head_6: 30,
            // チューニング用
            sub_name: None,
        };
    }
}

impl Evaluator {
    pub fn name(&self) -> String {
        let mut info = "Evaluator".to_owned();
        if let Some(extra) = &self.sub_name {
            info.push(' ');
            info.push_str(extra);
        } else {
            info.push_str(" Default");
        }
        info
    }

    pub fn short_name(&self) -> String {
        match &self.sub_name {
            Some(extra) => {
                let mut ret = extra.clone();
                ret.retain(|c| c != ' ');
                ret = ret.replace("#", "-");
                ret
            }
            None => "Default".into(),
        }
    }

    pub fn evaluate(&self, plan: &Plan) -> i32 {
        let cf = plan.field();
        let res = plan.rensa_result();

        if cf.is_dead() {
            return i32::MIN >> 7;
        }

        let mut score = 0_i32;

        {
            // 盤面
            for x in 1..=field::WIDTH {
                score += self.valley * cf.valley_depth(x) as i32;
                score += self.ridge * cf.ridge_height(x) as i32;
            }

            let average_height = average_height(cf);
            let mut diff_sum = 0.0;
            let mut diff_sq_sum = 0.0;
            for x in 1..=field::WIDTH {
                let ideal_height = average_height
                    + match x {
                        1 | 6 => 2.0,
                        3 | 4 => -2.0,
                        _ => 0.0,
                    };

                let diff = ideal_height - cf.height(x) as f32;
                diff_sum += diff.abs();
                diff_sq_sum += diff * diff;
            }
            let coef = if average_height < 1.0 {
                0.0
            } else if average_height < 3.0 {
                self.ideal_height_coef_1 as f32 / 1000.0
            } else if average_height < 5.0 {
                self.ideal_height_coef_2 as f32 / 1000.0
            } else if average_height < 7.0 {
                self.ideal_height_coef_3 as f32 / 1000.0
            } else if average_height < 9.0 {
                self.ideal_height_coef_4 as f32 / 1000.0
            } else {
                1.0
            };

            score += (self.ideal_height_diff as f32 * diff_sum * coef) as i32;
            score += (self.ideal_height_diff_sq as f32 * diff_sq_sum * coef) as i32;

            score += self.third_column_height * cf.height(3) as i32;
            score += self.third_column_height_sq * (cf.height(3) * cf.height(3)) as i32;

            score += self.unreachable_space * cf.count_unreachable_spaces() as i32;

            for x in 1..=field::WIDTH {
                if !cf.is_empty(x, 13) {
                    score += self.top_row[x - 1];
                }
            }
        }

        {
            // 連結
            let connectivity = connectivity(cf);
            score += self.connectivity_2 * connectivity[2];
            score += self.connectivity_3 * connectivity[3];
        }

        {
            // 発生した連鎖
            score += self.chain * res.chain as i32;
            score += self.chain_sq * (res.chain * res.chain) as i32;
            score += self.chain_score * (res.score / 1000) as i32;
            score += self.chain_frame * res.frame as i32;
        }

        {
            // 盤面から起こりうる連鎖
            let (main_chain, sub_chain) = detect_chains(cf);

            if let Some((rensa_result, ignition_y)) = main_chain {
                score += self.potential_main_chain * rensa_result.chain as i32;
                score +=
                    self.potential_main_chain * (rensa_result.chain * rensa_result.chain) as i32;
                score += self.potential_main_chain_frame * rensa_result.frame as i32;
                score += self.potential_main_chain_ignition_height * ignition_y as i32;
            }

            if let Some((rensa_result, ignition_y)) = sub_chain {
                score += self.potential_sub_chain * rensa_result.chain as i32;
                score +=
                    self.potential_sub_chain * (rensa_result.chain * rensa_result.chain) as i32;
                score += self.potential_sub_chain_frame * rensa_result.frame as i32;
                score += self.potential_sub_chain_ignition_height * ignition_y as i32;
            }
        }

        {
            // フレーム関係
            score += self.chigiri * plan.num_chigiri() as i32;
            score += self.move_frame * plan.frame() as i32;
        }

        {
            // パターンマッチング
            macro_rules! pattern_matching {
                ($name:ident) => {
                    score += self.$name * $name(cf) as i32;
                };
            }

            pattern_matching!(gtr_base_1);
            pattern_matching!(gtr_base_2);
            pattern_matching!(gtr_base_3);
            pattern_matching!(gtr_base_4);
            pattern_matching!(gtr_base_5);
            pattern_matching!(gtr_base_6);
            pattern_matching!(gtr_base_7);
            pattern_matching!(gtr_1);
            pattern_matching!(gtr_2);
            pattern_matching!(gtr_3);
            pattern_matching!(gtr_4);
            pattern_matching!(gtr_5);
            pattern_matching!(gtr_6);
            pattern_matching!(gtr_tail_1_1);
            pattern_matching!(gtr_tail_1_2);
            pattern_matching!(gtr_tail_1_3);
            pattern_matching!(gtr_tail_2_1);
            pattern_matching!(gtr_tail_2_2);
            pattern_matching!(gtr_tail_2_3);
            pattern_matching!(gtr_tail_2_4);
            pattern_matching!(gtr_tail_2_5);
            pattern_matching!(gtr_tail_2_6);
            pattern_matching!(gtr_tail_2_7);
            pattern_matching!(gtr_tail_3_1);
            pattern_matching!(gtr_tail_3_2);
            pattern_matching!(gtr_tail_3_3);
            pattern_matching!(gtr_tail_3_4);
            pattern_matching!(gtr_tail_4_1);
            pattern_matching!(gtr_tail_5_1);
            pattern_matching!(gtr_tail_5_2);
            pattern_matching!(gtr_tail_6_1);
            pattern_matching!(gtr_tail_6_2);
            pattern_matching!(gtr_tail_6_3);
            pattern_matching!(gtr_head_1);
            pattern_matching!(gtr_head_2);
            pattern_matching!(gtr_head_3);
            pattern_matching!(gtr_head_4);
            pattern_matching!(gtr_head_5);
            pattern_matching!(gtr_head_6);
        }

        score
    }
}

/// 各列の平均の高さを返す
fn average_height(cf: &CoreField) -> f32 {
    let mut sum = 0;
    for x in 1..=field::WIDTH {
        sum += cf.height(x);
    }
    sum as f32 / 6.0
}

/// 連結の数を数える
fn connectivity(cf: &CoreField) -> [i32; 4] {
    let mut con = [0; 4];
    for x in 1..=field::WIDTH {
        for y in 1..=cf.height(x) {
            // おじゃまなどは飛ばす
            if !cf.color(x, y).is_normal_color() {
                continue;
            }
            // TODO: すでに計算済みなら飛ばす（puyoai の `countConnectedPuyos`）
            let cnt = cf.count_connected(x, y);
            if cnt < 4 {
                con[cnt] += 1;
            }
        }
    }
    con
}

/// 与えられた盤面に対して、`detect_by_drop` で本線と副砲を検出する
/// - 複数あるなら、連鎖の効率（得点 / フレーム数）が一番良いものを選ぶ
/// - 本線は5000点以上の連鎖、副砲は5000点未満の連鎖とする
/// - 返り値は、本線と副砲に対する `(その連鎖の詳細, 発火点の高さ)`
fn detect_chains(cf: &CoreField) -> (Option<(RensaResult, usize)>, Option<(RensaResult, usize)>) {
    let mut main_chain: Option<(RensaResult, usize)> = None;
    let mut sub_chain: Option<(RensaResult, usize)> = None;

    detect_by_drop(
        &cf,
        &[false; 8],
        PurposeForFindingRensa::ForFire,
        2,
        13,
        |complemented_field: CoreField, cpl: &ColumnPuyoList| {
            // 同列に最大2個補完するので、`[0, 0, 2, 0, 0, 0]` のような感じになるはず
            let ignition_y = cf.height(
                (1..=field::WIDTH)
                    .max_by(|i, j| cpl.size_on(*i).cmp(&cpl.size_on(*j)))
                    .unwrap(),
            );

            let rensa_result = complemented_field.clone().simulate();
            let target_chain_opt = if rensa_result.score >= 5000 {
                &mut main_chain
            } else if rensa_result.score >= 70 {
                // おじゃまを少なくとも1個送れるなら副砲とみなす
                &mut sub_chain
            } else {
                return;
            };

            if let Some((ord_rensa_result, _ord_ignition_y)) = target_chain_opt {
                // TODO: 同率は処理する？（確率低すぎるのでしなくてよさそう）
                if ord_rensa_result.score * rensa_result.frame
                    < rensa_result.score * ord_rensa_result.frame
                {
                    *target_chain_opt = Some((rensa_result, ignition_y));
                }
            } else {
                *target_chain_opt = Some((rensa_result, ignition_y));
            }
        },
    );

    (main_chain, sub_chain)
}
