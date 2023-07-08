use onnxruntime::ndarray::{Array1, Array2};
use puyoai::{field::CoreField, kumipuyo::Kumipuyo};

use super::{feature::*, weight::*};

pub struct NdarrayEvaluator {
    input_linear: Array2<f32>,
    input_bias: Array1<f32>,
    hidden_linear: Array2<f32>,
    hidden_bias: Array1<f32>,
    output_linear: Array2<f32>,
    output_bias: Array1<f32>,
}

impl NdarrayEvaluator {
    pub fn new() -> Self {
        Self {
            input_linear: get_input_linear(),
            input_bias: get_input_bias(),
            hidden_linear: get_hidden_linear(),
            hidden_bias: get_hidden_bias(),
            output_linear: get_output_linear(),
            output_bias: get_output_bias(),
        }
    }

    pub fn evaluate(&self, cf: &CoreField, next1: &Kumipuyo, next2: &Kumipuyo) -> f32 {
        let input = convert_board_to_feature_indices(cf, next1, next2);
        let x = self.sparse_linear(&input, &self.input_linear);
        let x = x + &self.input_bias;
        let x = x.mapv(|v| v.max(0.0).min(1.0));
        let x = x.dot(&self.hidden_linear);
        let x = x + &self.hidden_bias;
        let x = x.mapv(|v| v.max(0.0).min(1.0));
        let x = x.dot(&self.output_linear);
        let x = x + &self.output_bias;
        x[0]
    }

    pub fn evaluate_slow(&self, cf: &CoreField, next1: &Kumipuyo, next2: &Kumipuyo) -> f32 {
        let input = convert_board_to_feature_ndarray(cf, next1, next2);
        let x = input.dot(&self.input_linear);
        let x = x + &self.input_bias;
        let x = x.mapv(|v| v.max(0.0).min(1.0));
        let x = x.dot(&self.hidden_linear);
        let x = x + &self.hidden_bias;
        let x = x.mapv(|v| v.max(0.0).min(1.0));
        let x = x.dot(&self.output_linear);
        let x = x + &self.output_bias;
        x[0]
    }

    fn sparse_linear(&self, feature_indices: &Vec<usize>, mat: &Array2<f32>) -> Array1<f32> {
        let dim = mat.shape()[1];
        let mut x = Array1::zeros(dim);
        for idx in feature_indices {
            x = x + mat.row(*idx);
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
        let evaluator = NdarrayEvaluator::new();

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
        }
    }
}
