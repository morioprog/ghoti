use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment, session::Session, tensor::OrtOwnedTensor, GraphOptimizationLevel,
    LoggingLevel,
};
use puyoai::{field::CoreField, kumipuyo::Kumipuyo};

use super::feature::*;

static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
    Environment::builder()
        .with_name(env!("CARGO_PKG_NAME"))
        .with_log_level(LoggingLevel::Info)
        .build()
        .unwrap()
});

pub struct NaiveEvaluator {
    _onnx_path: String,
    session: Session<'static>,
}

impl NaiveEvaluator {
    pub fn new(onnx_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let onnx_path = format!(
            "{}/src/evaluator/nnue/onnx/{}.onnx",
            env!("CARGO_MANIFEST_DIR"),
            onnx_name
        );
        let session = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::Basic)?
            .with_number_threads(1)?
            .with_model_from_file(onnx_path.clone())?;

        let input_shape: Vec<usize> = session.inputs[0].dimensions().map(|d| d.unwrap()).collect();
        assert_eq!(input_shape, vec![FEATURE_SIZE]);

        Ok(NaiveEvaluator {
            _onnx_path: onnx_path,
            session,
        })
    }

    pub fn evaluate(&mut self, cf: &CoreField, next1: &Kumipuyo, next2: &Kumipuyo) -> f32 {
        let input = convert_board_to_feature(cf, next1, next2);
        self.evaluate_from_array(&input)
    }

    pub fn evaluate_from_array(&mut self, input: &[f32; FEATURE_SIZE]) -> f32 {
        let outputs: Vec<OrtOwnedTensor<f32, _>> =
            self.session.run(vec![input.to_vec().into()]).unwrap();
        outputs[0][0]
    }
}

#[cfg(test)]
mod tests {
    use puyoai::color::PuyoColor;

    use super::*;

    #[test]
    fn test_evaluator() {
        let onnx_name = "data=660000-lr=1e-4";
        let mut evaluator = NaiveEvaluator::new(onnx_name).unwrap();

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
