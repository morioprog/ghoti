use puyoai::{field::CoreField, kumipuyo::Kumipuyo};

use super::{feature::*, weight_nalgebra::*};

pub struct NalgebraEvaluator {
    input_bias: InputBiasDim,
    hidden_linear: HiddenLinearDim,
    hidden_bias: HiddenBiasDim,
    output_linear: OutputLinearDim,
    output_bias: OutputBiasDim,
}

impl NalgebraEvaluator {
    pub fn new() -> Self {
        Self {
            input_bias: get_input_bias(),
            hidden_linear: get_hidden_linear(),
            hidden_bias: get_hidden_bias(),
            output_linear: get_output_linear(),
            output_bias: get_output_bias(),
        }
    }

    pub fn evaluate(&self, cf: &CoreField, next1: &Kumipuyo, next2: &Kumipuyo) -> f32 {
        let input = convert_board_to_feature_indices(cf, next1, next2);
        let x = self.sparse_linear(&input);
        let x = x + &self.input_bias;
        let x = x.map(|v| v.max(0.0).min(1.0));
        let x = x * &self.hidden_linear;
        let x = x + &self.hidden_bias;
        let x = x.map(|v| v.max(0.0).min(1.0));
        let x = x * &self.output_linear;
        let x = x + &self.output_bias;
        x[0]
    }

    fn sparse_linear(&self, feature_indices: &Vec<usize>) -> InputBiasDim {
        let mut x = InputBiasDim::zeros();
        for idx in feature_indices {
            x += InputBiasDim::from_row_slice(INPUT_LINEAR[*idx]);
        }
        x
    }
}

#[cfg(test)]
mod tests {
    use puyoai::color::PuyoColor;

    use super::*;

    #[test]
    fn test_evaluator() {
        let evaluator = NalgebraEvaluator::new();

        {
            // rggg/bb/g/b/r/ybr/,gr,rb
            let cf = CoreField::from_str(concat!(
                "G.....", // 4
                "G....R", // 3
                "GB...B", // 2
                "RBGBRY"  // 1
            ));
            let next1 = Kumipuyo::new(PuyoColor::GREEN, PuyoColor::RED);
            let next2 = Kumipuyo::new(PuyoColor::RED, PuyoColor::BLUE);
            assert!((evaluator.evaluate(&cf, &next1, &next2) - 0.125).abs() <= 1e-3);
            // assert_eq!(evaluator.evaluate(&cf, &next1, &next2), 0.0);
        }
    }
}
