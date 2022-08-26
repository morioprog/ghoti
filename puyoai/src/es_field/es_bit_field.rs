use puyoai_core::{
    field::BitField,
    field_bit::FieldBit,
    rensa_result::RensaResult,
    rensa_tracker::{RensaNonTracker, RensaTracker},
};

use crate::es_frame;

pub trait EsBitField {
    fn es_simulate(&mut self) -> RensaResult;
    fn es_simulate_with_tracker<T: RensaTracker>(&mut self, tracker: &mut T) -> RensaResult;
    fn es_simulate_from_middle(&mut self, current_chain: usize) -> RensaResult;
    fn es_simulate_from_middle_with_tracker<T: RensaTracker>(
        &mut self,
        tracker: &mut T,
        current_chain: usize,
    ) -> RensaResult;
}

#[cfg(all(target_feature = "avx2", target_feature = "bmi2"))]
impl EsBitField for BitField {
    fn es_simulate(&mut self) -> RensaResult {
        let mut tracker = RensaNonTracker::new();
        self.es_simulate_with_tracker(&mut tracker)
    }

    fn es_simulate_with_tracker<T: RensaTracker>(&mut self, tracker: &mut T) -> RensaResult {
        self.es_simulate_from_middle_with_tracker(tracker, 1)
    }

    fn es_simulate_from_middle(&mut self, current_chain: usize) -> RensaResult {
        let mut tracker = RensaNonTracker::new();
        self.es_simulate_from_middle_with_tracker(&mut tracker, current_chain)
    }

    fn es_simulate_from_middle_with_tracker<T: RensaTracker>(
        &mut self,
        tracker: &mut T,
        current_chain: usize,
    ) -> RensaResult {
        let escaped = self.escape_invisible();

        let mut score = 0;
        let mut frames = 0;
        let mut quick = false;
        let mut current_chain = current_chain;

        loop {
            let mut erased = unsafe { FieldBit::uninitialized() };
            let nth_chain_score = self.vanish(current_chain, &mut erased, tracker);
            if nth_chain_score == 0 {
                break;
            }

            current_chain += 1;
            score += nth_chain_score;

            let max_drops = unsafe { self.drop_after_vanish(erased, tracker) };
            frames += es_frame::FRAMES_CHAIN[max_drops];
            if max_drops == 0 {
                quick = true;
            }
        }

        self.recover_invisible(&escaped);
        RensaResult::new(current_chain - 1, score, frames, quick)
    }
}

#[cfg(all(test, target_feature = "avx2", target_feature = "bmi2"))]
mod tests_simulation {
    use puyoai_core::field::BitField;

    use super::*;
    use crate::es_frame;

    #[test]
    fn test_es_simulate() {
        struct SimulationTestcase {
            field: BitField,
            chain: usize,
            score: usize,
            frame: usize,
            quick: bool,
        }

        let simulation_testcases = &[
            SimulationTestcase {
                field: BitField::from_str(concat!(".BBBB.")),
                chain: 1,
                score: 40,
                frame: es_frame::FRAMES_CHAIN[0],
                quick: true,
            },
            SimulationTestcase {
                field: BitField::from_str(concat!(
                    ".RBRB.", // 4
                    "RBRBR.", // 3
                    "RBRBR.", // 2
                    "RBRBRR"  // 1
                )),
                chain: 5,
                score: 40 + 40 * 8 + 40 * 16 + 40 * 32 + 40 * 64,
                frame: es_frame::FRAMES_CHAIN[3] // 1連鎖目
                    + es_frame::FRAMES_CHAIN[3]  // 2連鎖目
                    + es_frame::FRAMES_CHAIN[3]  // 3連鎖目
                    + es_frame::FRAMES_CHAIN[3]  // 4連鎖目
                    + es_frame::FRAMES_CHAIN[0], // 5連鎖目
                quick: true,
            },
            SimulationTestcase {
                field: BitField::from_str(concat!(
                    ".YGGY.", // 4
                    "BBBBBB", // 3
                    "GYBBYG", // 2
                    "BBBBBB"  // 1
                )),
                chain: 1,
                score: 140 * 10,
                frame: es_frame::FRAMES_CHAIN[3],
                quick: false,
            },
        ];

        for testcase in simulation_testcases {
            let mut bf = testcase.field.clone();
            let rensa_result = bf.es_simulate();
            assert_eq!(testcase.chain, rensa_result.chain);
            assert_eq!(testcase.score, rensa_result.score);
            assert_eq!(testcase.frame, rensa_result.frame);
            assert_eq!(testcase.quick, rensa_result.quick);
        }
    }

    #[test]
    fn test_es_simulate_from_middle() {
        struct SimulationTestcase {
            field: BitField,
            current_chain: usize,
            chain: usize,
            score: usize,
            frame: usize,
            quick: bool,
        }

        let simulation_testcases = &[
            SimulationTestcase {
                field: BitField::from_str(concat!(".BBBB.")),
                current_chain: 3,
                chain: 3,
                score: 40 * 16,
                frame: es_frame::FRAMES_CHAIN[0],
                quick: true,
            },
            SimulationTestcase {
                field: BitField::from_str(concat!(
                    ".RBRB.", // 4
                    "RBRBR.", // 3
                    "RBRBR.", // 2
                    "RBRBRR"  // 1
                )),
                current_chain: 2,
                chain: 6,
                score: 40 * 8 + 40 * 16 + 40 * 32 + 40 * 64 + 40 * 96,
                frame: es_frame::FRAMES_CHAIN[3] // 2連鎖目
                    + es_frame::FRAMES_CHAIN[3]  // 3連鎖目
                    + es_frame::FRAMES_CHAIN[3]  // 4連鎖目
                    + es_frame::FRAMES_CHAIN[3]  // 5連鎖目
                    + es_frame::FRAMES_CHAIN[0], // 6連鎖目
                quick: true,
            },
            SimulationTestcase {
                field: BitField::from_str(concat!(
                    ".YGGY.", // 4
                    "BBBBBB", // 3
                    "GYBBYG", // 2
                    "BBBBBB"  // 1
                )),
                current_chain: 1,
                chain: 1,
                score: 140 * 10,
                frame: es_frame::FRAMES_CHAIN[3],
                quick: false,
            },
        ];

        for testcase in simulation_testcases {
            let mut bf = testcase.field.clone();
            let rensa_result = bf.es_simulate_from_middle(testcase.current_chain);
            assert_eq!(testcase.chain, rensa_result.chain);
            assert_eq!(testcase.score, rensa_result.score);
            assert_eq!(testcase.frame, rensa_result.frame);
            assert_eq!(testcase.quick, rensa_result.quick);
        }
    }
}
