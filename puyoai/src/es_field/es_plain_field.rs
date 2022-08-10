use puyoai_core::{color::Color, field::plain_field::PlainField, rensa_result::RensaResult};

use crate::es_frame;

pub trait EsPlainField {
    fn es_simulate(&mut self) -> RensaResult;
}

impl<C: Color> EsPlainField for PlainField<C> {
    fn es_simulate(&mut self) -> RensaResult {
        let mut chains = 1;
        let mut score = 0;
        let mut frames = 0;
        let mut quick = false;

        loop {
            let nth_chain_score = self.vanish(chains);
            if nth_chain_score == 0 {
                break;
            }

            chains += 1;
            score += nth_chain_score;

            let max_drops = self.drop();
            frames += es_frame::FRAMES_CHAIN[max_drops];
            if max_drops == 0 {
                quick = true;
                break;
            }
        }

        RensaResult::new(chains - 1, score, frames, quick)
    }
}

#[cfg(test)]
mod tests {
    use puyoai_core::field::PuyoPlainField;

    use super::*;
    use crate::es_frame;

    struct SimulationTestcase {
        field: PuyoPlainField,
        chain: usize,
        score: usize,
        frame: usize,
        quick: bool,
    }

    #[test]
    fn test_es_simulate() {
        let simulation_testcases = &[
            SimulationTestcase {
                field: PuyoPlainField::from_str(concat!(".BBBB.")),
                chain: 1,
                score: 40,
                frame: es_frame::FRAMES_CHAIN[0],
                quick: true,
            },
            SimulationTestcase {
                field: PuyoPlainField::from_str(concat!(
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
                field: PuyoPlainField::from_str(concat!(
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
            let mut pf = testcase.field.clone();
            let rensa_result = pf.es_simulate();
            assert_eq!(testcase.chain, rensa_result.chain);
            assert_eq!(testcase.score, rensa_result.score);
            assert_eq!(testcase.frame, rensa_result.frame);
            assert_eq!(testcase.quick, rensa_result.quick);
        }
    }
}
