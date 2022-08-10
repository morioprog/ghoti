use cpu::evaluator::Evaluator;
use rand::prelude::*;

pub trait Mutateable: Default {
    fn generate(sub_name: String) -> Self;
    fn crossover(parent1: &Self, parent2: &Self, sub_name: String) -> Self;
    fn name(&self) -> &str;
}

impl Mutateable for Evaluator {
    fn generate(sub_name: String) -> Self {
        Evaluator {
            // 盤面
            valley: thread_rng().gen_range(-999..1000),
            ridge: thread_rng().gen_range(-999..1000),
            ideal_height_diff: thread_rng().gen_range(-999..1000),
            ideal_height_diff_sq: thread_rng().gen_range(-999..1000),
            ideal_height_coef_1: thread_rng().gen_range(-999..1000),
            ideal_height_coef_2: thread_rng().gen_range(-999..1000),
            ideal_height_coef_3: thread_rng().gen_range(-999..1000),
            ideal_height_coef_4: thread_rng().gen_range(-999..1000),
            third_column_height: thread_rng().gen_range(-999..1000),
            third_column_height_sq: thread_rng().gen_range(-999..1000),
            unreachable_space: thread_rng().gen_range(-999..1000),
            top_row: [
                thread_rng().gen_range(-999..1000),
                thread_rng().gen_range(-999..1000),
                thread_rng().gen_range(-999..1000),
                thread_rng().gen_range(-999..1000),
                thread_rng().gen_range(-999..1000),
                thread_rng().gen_range(-999..1000),
            ],
            // 連結
            connectivity_2: thread_rng().gen_range(-999..1000),
            connectivity_3: thread_rng().gen_range(-999..1000),
            // 発生した連鎖
            chain: thread_rng().gen_range(-999..1000),
            chain_sq: thread_rng().gen_range(-999..1000),
            chain_score: thread_rng().gen_range(-999..1000),
            chain_frame: thread_rng().gen_range(-999..1000),
            // 盤面から起こりうる連鎖
            potential_main_chain: thread_rng().gen_range(-999..1000),
            potential_main_chain_sq: thread_rng().gen_range(-999..1000),
            potential_main_chain_frame: thread_rng().gen_range(-999..1000),
            potential_main_chain_ignition_height: thread_rng().gen_range(-999..1000),
            potential_sub_chain: thread_rng().gen_range(-999..1000),
            potential_sub_chain_sq: thread_rng().gen_range(-999..1000),
            potential_sub_chain_frame: thread_rng().gen_range(-999..1000),
            potential_sub_chain_ignition_height: thread_rng().gen_range(-999..1000),
            // フレーム関係
            chigiri: thread_rng().gen_range(-999..1000),
            move_frame: thread_rng().gen_range(-999..1000),
            // パターンマッチング
            gtr_base_1: thread_rng().gen_range(-999..1000),
            gtr_base_2: thread_rng().gen_range(-999..1000),
            gtr_base_3: thread_rng().gen_range(-999..1000),
            gtr_base_4: thread_rng().gen_range(-999..1000),
            gtr_base_5: thread_rng().gen_range(-999..1000),
            gtr_base_6: thread_rng().gen_range(-999..1000),
            gtr_base_7: thread_rng().gen_range(-999..1000),
            gtr_1: thread_rng().gen_range(-999..1000),
            gtr_2: thread_rng().gen_range(-999..1000),
            gtr_3: thread_rng().gen_range(-999..1000),
            gtr_4: thread_rng().gen_range(-999..1000),
            gtr_5: thread_rng().gen_range(-999..1000),
            gtr_6: thread_rng().gen_range(-999..1000),
            gtr_tail_1_1: thread_rng().gen_range(-999..1000),
            gtr_tail_1_2: thread_rng().gen_range(-999..1000),
            gtr_tail_1_3: thread_rng().gen_range(-999..1000),
            gtr_tail_2_1: thread_rng().gen_range(-999..1000),
            gtr_tail_2_2: thread_rng().gen_range(-999..1000),
            gtr_tail_2_3: thread_rng().gen_range(-999..1000),
            gtr_tail_2_4: thread_rng().gen_range(-999..1000),
            gtr_tail_2_5: thread_rng().gen_range(-999..1000),
            gtr_tail_2_6: thread_rng().gen_range(-999..1000),
            gtr_tail_2_7: thread_rng().gen_range(-999..1000),
            gtr_tail_3_1: thread_rng().gen_range(-999..1000),
            gtr_tail_3_2: thread_rng().gen_range(-999..1000),
            gtr_tail_3_3: thread_rng().gen_range(-999..1000),
            gtr_tail_3_4: thread_rng().gen_range(-999..1000),
            gtr_tail_4_1: thread_rng().gen_range(-999..1000),
            gtr_tail_5_1: thread_rng().gen_range(-999..1000),
            gtr_tail_5_2: thread_rng().gen_range(-999..1000),
            gtr_tail_6_1: thread_rng().gen_range(-999..1000),
            gtr_tail_6_2: thread_rng().gen_range(-999..1000),
            gtr_tail_6_3: thread_rng().gen_range(-999..1000),
            gtr_head_1: thread_rng().gen_range(-999..1000),
            gtr_head_2: thread_rng().gen_range(-999..1000),
            gtr_head_3: thread_rng().gen_range(-999..1000),
            gtr_head_4: thread_rng().gen_range(-999..1000),
            gtr_head_5: thread_rng().gen_range(-999..1000),
            gtr_head_6: thread_rng().gen_range(-999..1000),
            // チューニング用
            sub_name: Some(sub_name),
        }
    }

    fn crossover(parent1: &Self, parent2: &Self, sub_name: String) -> Self {
        Evaluator {
            // 盤面
            valley: crossover_gene(parent1.valley, parent2.valley),
            ridge: crossover_gene(parent1.ridge, parent2.ridge),
            ideal_height_diff: crossover_gene(parent1.ideal_height_diff, parent2.ideal_height_diff),
            ideal_height_diff_sq: crossover_gene(
                parent1.ideal_height_diff_sq,
                parent2.ideal_height_diff_sq,
            ),
            ideal_height_coef_1: crossover_gene(
                parent1.ideal_height_coef_1,
                parent2.ideal_height_coef_1,
            ),
            ideal_height_coef_2: crossover_gene(
                parent1.ideal_height_coef_2,
                parent2.ideal_height_coef_2,
            ),
            ideal_height_coef_3: crossover_gene(
                parent1.ideal_height_coef_3,
                parent2.ideal_height_coef_3,
            ),
            ideal_height_coef_4: crossover_gene(
                parent1.ideal_height_coef_4,
                parent2.ideal_height_coef_4,
            ),
            third_column_height: crossover_gene(
                parent1.third_column_height,
                parent2.third_column_height,
            ),
            third_column_height_sq: crossover_gene(
                parent1.third_column_height_sq,
                parent2.third_column_height_sq,
            ),
            unreachable_space: crossover_gene(parent1.unreachable_space, parent2.unreachable_space),
            top_row: [
                crossover_gene(parent1.top_row[0], parent2.top_row[0]),
                crossover_gene(parent1.top_row[1], parent2.top_row[1]),
                crossover_gene(parent1.top_row[2], parent2.top_row[2]),
                crossover_gene(parent1.top_row[3], parent2.top_row[3]),
                crossover_gene(parent1.top_row[4], parent2.top_row[4]),
                crossover_gene(parent1.top_row[5], parent2.top_row[5]),
            ],
            // 連結
            connectivity_2: crossover_gene(parent1.connectivity_2, parent2.connectivity_2),
            connectivity_3: crossover_gene(parent1.connectivity_3, parent2.connectivity_3),
            // 発生した連鎖
            chain: crossover_gene(parent1.chain, parent2.chain),
            chain_sq: crossover_gene(parent1.chain_sq, parent2.chain_sq),
            chain_score: crossover_gene(parent1.chain_score, parent2.chain_score),
            chain_frame: crossover_gene(parent1.chain_frame, parent2.chain_frame),
            // 盤面から起こりうる連鎖
            potential_main_chain: crossover_gene(
                parent1.potential_main_chain,
                parent2.potential_main_chain,
            ),
            potential_main_chain_sq: crossover_gene(
                parent1.potential_main_chain_sq,
                parent2.potential_main_chain_sq,
            ),
            potential_main_chain_frame: crossover_gene(
                parent1.potential_main_chain_frame,
                parent2.potential_main_chain_frame,
            ),
            potential_main_chain_ignition_height: crossover_gene(
                parent1.potential_main_chain_ignition_height,
                parent2.potential_main_chain_ignition_height,
            ),
            potential_sub_chain: crossover_gene(
                parent1.potential_sub_chain,
                parent2.potential_sub_chain,
            ),
            potential_sub_chain_sq: crossover_gene(
                parent1.potential_sub_chain_sq,
                parent2.potential_sub_chain_sq,
            ),
            potential_sub_chain_frame: crossover_gene(
                parent1.potential_sub_chain_frame,
                parent2.potential_sub_chain_frame,
            ),
            potential_sub_chain_ignition_height: crossover_gene(
                parent1.potential_sub_chain_ignition_height,
                parent2.potential_sub_chain_ignition_height,
            ),
            // フレーム関係
            chigiri: crossover_gene(parent1.chigiri, parent2.chigiri),
            move_frame: crossover_gene(parent1.move_frame, parent2.move_frame),
            // パターンマッチング
            gtr_base_1: crossover_gene(parent1.gtr_base_1, parent2.gtr_base_1),
            gtr_base_2: crossover_gene(parent1.gtr_base_2, parent2.gtr_base_2),
            gtr_base_3: crossover_gene(parent1.gtr_base_3, parent2.gtr_base_3),
            gtr_base_4: crossover_gene(parent1.gtr_base_4, parent2.gtr_base_4),
            gtr_base_5: crossover_gene(parent1.gtr_base_5, parent2.gtr_base_5),
            gtr_base_6: crossover_gene(parent1.gtr_base_6, parent2.gtr_base_6),
            gtr_base_7: crossover_gene(parent1.gtr_base_7, parent2.gtr_base_7),
            gtr_1: crossover_gene(parent1.gtr_1, parent2.gtr_1),
            gtr_2: crossover_gene(parent1.gtr_2, parent2.gtr_2),
            gtr_3: crossover_gene(parent1.gtr_3, parent2.gtr_3),
            gtr_4: crossover_gene(parent1.gtr_4, parent2.gtr_4),
            gtr_5: crossover_gene(parent1.gtr_5, parent2.gtr_5),
            gtr_6: crossover_gene(parent1.gtr_6, parent2.gtr_6),
            gtr_tail_1_1: crossover_gene(parent1.gtr_tail_1_1, parent2.gtr_tail_1_1),
            gtr_tail_1_2: crossover_gene(parent1.gtr_tail_1_2, parent2.gtr_tail_1_2),
            gtr_tail_1_3: crossover_gene(parent1.gtr_tail_1_3, parent2.gtr_tail_1_3),
            gtr_tail_2_1: crossover_gene(parent1.gtr_tail_2_1, parent2.gtr_tail_2_1),
            gtr_tail_2_2: crossover_gene(parent1.gtr_tail_2_2, parent2.gtr_tail_2_2),
            gtr_tail_2_3: crossover_gene(parent1.gtr_tail_2_3, parent2.gtr_tail_2_3),
            gtr_tail_2_4: crossover_gene(parent1.gtr_tail_2_4, parent2.gtr_tail_2_4),
            gtr_tail_2_5: crossover_gene(parent1.gtr_tail_2_5, parent2.gtr_tail_2_5),
            gtr_tail_2_6: crossover_gene(parent1.gtr_tail_2_6, parent2.gtr_tail_2_6),
            gtr_tail_2_7: crossover_gene(parent1.gtr_tail_2_7, parent2.gtr_tail_2_7),
            gtr_tail_3_1: crossover_gene(parent1.gtr_tail_3_1, parent2.gtr_tail_3_1),
            gtr_tail_3_2: crossover_gene(parent1.gtr_tail_3_2, parent2.gtr_tail_3_2),
            gtr_tail_3_3: crossover_gene(parent1.gtr_tail_3_3, parent2.gtr_tail_3_3),
            gtr_tail_3_4: crossover_gene(parent1.gtr_tail_3_4, parent2.gtr_tail_3_4),
            gtr_tail_4_1: crossover_gene(parent1.gtr_tail_4_1, parent2.gtr_tail_4_1),
            gtr_tail_5_1: crossover_gene(parent1.gtr_tail_5_1, parent2.gtr_tail_5_1),
            gtr_tail_5_2: crossover_gene(parent1.gtr_tail_5_2, parent2.gtr_tail_5_2),
            gtr_tail_6_1: crossover_gene(parent1.gtr_tail_6_1, parent2.gtr_tail_6_1),
            gtr_tail_6_2: crossover_gene(parent1.gtr_tail_6_2, parent2.gtr_tail_6_2),
            gtr_tail_6_3: crossover_gene(parent1.gtr_tail_6_3, parent2.gtr_tail_6_3),
            gtr_head_1: crossover_gene(parent1.gtr_head_1, parent2.gtr_head_1),
            gtr_head_2: crossover_gene(parent1.gtr_head_2, parent2.gtr_head_2),
            gtr_head_3: crossover_gene(parent1.gtr_head_3, parent2.gtr_head_3),
            gtr_head_4: crossover_gene(parent1.gtr_head_4, parent2.gtr_head_4),
            gtr_head_5: crossover_gene(parent1.gtr_head_5, parent2.gtr_head_5),
            gtr_head_6: crossover_gene(parent1.gtr_head_6, parent2.gtr_head_6),
            // チューニング用
            sub_name: Some(sub_name),
        }
    }

    fn name(&self) -> &str {
        self.sub_name.as_ref().map(|s| &**s).unwrap_or("")
    }
}

fn crossover_gene(v1: i32, v2: i32) -> i32 {
    let v = match thread_rng().gen_range(0..100) {
        0..=41 => v1,             // 42%
        42..=83 => v2,            // 42%
        84..=98 => (v1 + v2) / 2, // 15%
        _ => thread_rng().gen_range(-999..1000),
    } + thread_rng().gen_range(-10..11);
    if v < -999 {
        -999
    } else if v > 999 {
        999
    } else {
        v
    }
}
