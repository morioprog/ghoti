use puyoai_core::{
    color::PuyoColor,
    decision::Decision,
    field::{self, CoreField},
    rensa_result::RensaResult,
};
use rand::seq::SliceRandom;

use super::EsBitField;
use crate::es_frame;

pub trait EsCoreField {
    fn es_simulate(&mut self) -> RensaResult;
    fn es_simulate_from_middle(&mut self, current_chain: usize) -> RensaResult;
    fn es_frames_to_drop_next(&self, decision: &Decision) -> usize;
    fn es_drop_ojama(&mut self, ojama: usize, seed: Option<u8>) -> usize;
}

impl EsCoreField for CoreField {
    fn es_simulate(&mut self) -> RensaResult {
        let result = self.field_mut().es_simulate();

        self.update_height();

        result
    }

    fn es_simulate_from_middle(&mut self, current_chain: usize) -> RensaResult {
        let result = self.field_mut().es_simulate_from_middle(current_chain);

        self.update_height();

        result
    }

    fn es_frames_to_drop_next(&self, decision: &Decision) -> usize {
        let x1 = decision.axis_x();
        let x2 = decision.child_x();

        let h1 = self.height(x1);
        let h2 = self.height(x2);

        // 何回横移動が必要か
        let diff_row3 = (3 - x1 as isize).abs() as usize;

        // 各マスごとの接地までにかかるフレーム数
        let mut drop_frames = es_frame::FRAMES_TO_MOVE_HORIZONTALLY[diff_row3]
            + if decision.rot() == 0 {
                es_frame::FRAMES_GROUNDING[h1]
            } else if decision.rot() == 2 {
                es_frame::FRAMES_GROUNDING[h1 + 1]
            } else {
                // 子ぷよを接地して軸ぷよをちぎる場合は、子ぷよ側の高さに準拠
                es_frame::FRAMES_GROUNDING[h1.max(h2)]
            };

        // ちぎりにかかるフレーム数
        drop_frames += es_frame::FRAMES_CHIGIRI[(h1 as isize - h2 as isize).abs() as usize];

        // 上部でのちぎりは回転モーション中にちぎりモーションが発生します。
        // 以下のマスに軸ぷよがあるちぎり方をする場合、ちぎりのフレーム数に足してください。
        // [2, 4, 6, 4, 2, *] // 12列目
        // [*, 2, 4, 2, *, *] // 11列目
        // [*, *, 2, *, *, *] // 10列目
        // TODO: field::HEIGHT で 8 を表して一般化？
        if h1 != h2 && h1 > 8 + diff_row3 {
            drop_frames += (h1 - 8 - diff_row3) << 1;
        }

        // 壁越えの操作は時間がかかるので、ペナルティを課す
        // TODO: 他にも壁越えになりうるパターンがあるかも
        if h1 > field::HEIGHT
            || ((x1 != 3 || x2 != 3)
                && self.height(2) >= field::HEIGHT
                && self.height(4) >= field::HEIGHT)
        {
            drop_frames += es_frame::FRAMES_KABEGOE;
        }

        drop_frames
    }

    /// 盤面におじゃまを落として、それにかかるフレーム数を返す
    fn es_drop_ojama(&mut self, ojama: usize, seed: Option<u8>) -> usize {
        if ojama == 0 {
            return 0;
        }

        // 最大赤玉1つ（30個）
        assert!(ojama <= 30);

        // おじゃま数による硬直フレーム数
        let ojama_freeze_frame = es_frame::FRAMES_GROUNDING_OJAMA_QUANTITY[ojama];
        // おじゃまの落下に要するフレーム数
        let mut ojama_drop_frame = 0;

        // まず列を落とす
        let row = ojama / field::WIDTH;
        let ojama = ojama % field::WIDTH;
        for x in 1..=field::WIDTH {
            for _ in 0..row {
                self.drop_puyo_on_with_max_height(x, PuyoColor::OJAMA, 14);

                let height = self.height(x);
                ojama_drop_frame = std::cmp::max(
                    ojama_drop_frame,
                    es_frame::FRAMES_GROUNDING_OJAMA_POSITION[height][x],
                );
            }
        }

        // 端数を落とす
        if ojama > 0 {
            let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed(
                [match seed {
                    Some(seed) => seed,
                    None => ojama as u8,
                }; 32],
            );
            let idxs: Vec<usize> = (1..=field::WIDTH).collect();
            let chose: Vec<_> = idxs.choose_multiple::<_>(&mut rng, ojama).collect();
            for x in chose {
                self.drop_puyo_on_with_max_height(*x, PuyoColor::OJAMA, 14);

                let height = self.height(*x);
                ojama_drop_frame = std::cmp::max(
                    ojama_drop_frame,
                    es_frame::FRAMES_GROUNDING_OJAMA_POSITION[height][*x],
                );
            }
        }

        // おじゃまが落ちた分、高さを再計算する
        self.update_height();

        ojama_freeze_frame + ojama_drop_frame
    }
}

#[cfg(test)]
mod tests {
    use puyoai_core::{decision::Decision, field::CoreField};

    use super::*;
    use crate::es_frame;

    #[test]
    fn test_es_frames_to_drop_next_without_chigiri() {
        let cf = CoreField::new();

        assert_eq!(
            es_frame::FRAMES_GROUNDING[0],
            cf.es_frames_to_drop_next(&Decision::new(3, 0))
        );
        assert_eq!(
            es_frame::FRAMES_GROUNDING[0],
            cf.es_frames_to_drop_next(&Decision::new(3, 1))
        );
        assert_eq!(
            es_frame::FRAMES_GROUNDING[1],
            cf.es_frames_to_drop_next(&Decision::new(3, 2))
        );
        assert_eq!(
            es_frame::FRAMES_GROUNDING[0],
            cf.es_frames_to_drop_next(&Decision::new(3, 3))
        );
        assert_eq!(
            es_frame::FRAMES_GROUNDING[0] + es_frame::FRAMES_TO_MOVE_HORIZONTALLY[2],
            cf.es_frames_to_drop_next(&Decision::new(1, 0))
        );
    }

    #[test]
    fn test_es_frames_to_drop_next_with_chigiri() {
        let cf = CoreField::from_str(concat!(
            "..O...", // 4
            "..O...", // 3
            "..O...", // 2
            "..O..."  // 1
        ));

        assert_eq!(
            es_frame::FRAMES_GROUNDING[4] + es_frame::FRAMES_CHIGIRI[4],
            cf.es_frames_to_drop_next(&Decision::new(3, 1))
        );
        assert_eq!(
            es_frame::FRAMES_GROUNDING[4]
                + es_frame::FRAMES_TO_MOVE_HORIZONTALLY[1]
                + es_frame::FRAMES_CHIGIRI[4],
            cf.es_frames_to_drop_next(&Decision::new(2, 1))
        );
    }

    #[test]
    fn test_es_frames_to_drop_next_on_13th_row() {
        let cf = CoreField::from_str(concat!(
            "OO.OOO", // 12
            "OOOOOO", // 11
            "OOOOOO", // 10
            "OOOOOO", // 9
            "OOOOOO", // 8
            "OOOOOO", // 7
            "OOOOOO", // 6
            "OOOOOO", // 5
            "OOOOOO", // 4
            "OOOOOO", // 3
            "OOOOOO", // 2
            "OOOOOO"  // 1
        ));

        assert_eq!(11, cf.height(3));
        assert_eq!(12, cf.height(4));

        // We cannot put with Decision(4, 2).

        assert_eq!(
            es_frame::FRAMES_GROUNDING[12]
                + es_frame::FRAMES_TO_MOVE_HORIZONTALLY[1]
                + es_frame::FRAMES_KABEGOE,
            cf.es_frames_to_drop_next(&Decision::new(4, 0))
        );
        assert_eq!(
            es_frame::FRAMES_GROUNDING[12]
                + es_frame::FRAMES_TO_MOVE_HORIZONTALLY[1]
                + es_frame::FRAMES_KABEGOE,
            cf.es_frames_to_drop_next(&Decision::new(4, 1))
        );
        assert_eq!(
            es_frame::FRAMES_GROUNDING[12]
                + es_frame::FRAMES_TO_MOVE_HORIZONTALLY[1]
                + es_frame::FRAMES_CHIGIRI[1]
                + es_frame::FRAMES_KABEGOE
                + 6, // 回転モーション中にちぎりモーションが発生
            cf.es_frames_to_drop_next(&Decision::new(4, 3))
        );
    }
}
