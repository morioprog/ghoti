use once_cell::sync::Lazy;
use onnxruntime::{
    environment::Environment, session::Session, tensor::OrtOwnedTensor, GraphOptimizationLevel,
    LoggingLevel,
};
use puyoai::{
    color::PuyoColor,
    field::{self, CoreField},
    kumipuyo::Kumipuyo,
};

static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
    Environment::builder()
        .with_name(env!("CARGO_PKG_NAME"))
        .with_log_level(LoggingLevel::Info)
        .build()
        .unwrap()
});

// TODO: 他の特徴も試しやすくする
const FEATURE_SIZE: usize = 3316;

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

fn convert_board_to_feature(
    cf: &CoreField,
    next1: &Kumipuyo,
    next2: &Kumipuyo,
) -> [f32; FEATURE_SIZE] {
    #[inline]
    fn index_margin(idx: usize) -> usize {
        idx * (159 - idx) >> 1
    }
    #[inline]
    fn color_to_index(color: PuyoColor) -> usize {
        match color {
            PuyoColor::RED => 0,
            PuyoColor::BLUE => 1,
            PuyoColor::YELLOW => 2,
            PuyoColor::GREEN => 3,
            _ => unreachable!(),
        }
    }

    let mut feature = [0.0; FEATURE_SIZE];

    let mut color_indices: [Vec<usize>; 4] = Default::default();
    color_indices[color_to_index(next1.axis())].push(0);
    color_indices[color_to_index(next1.child())].push(1);
    color_indices[color_to_index(next2.axis())].push(2);
    color_indices[color_to_index(next2.child())].push(3);

    for y in 1..=field::HEIGHT + 1 {
        let mut updated = false;
        for x in 1..=field::WIDTH {
            let index = (y - 1) * field::WIDTH + (x - 1);
            let color = cf.color(x, y);
            match color {
                PuyoColor::EMPTY => continue,
                PuyoColor::OJAMA => {
                    feature[3160 + index] = 1.0;
                }
                PuyoColor::RED | PuyoColor::BLUE | PuyoColor::YELLOW | PuyoColor::GREEN => {
                    let color_index = color_to_index(color);
                    color_indices[color_index].push(index + 4);
                }
                _ => unreachable!(),
            }
            updated = true;
        }
        if !updated {
            break;
        }
    }

    for l in color_indices.iter() {
        if l.len() == 1 {
            feature[3236 + l[0]] = 1.0;
        } else {
            for i in 0..l.len() {
                for j in i + 1..l.len() {
                    let index = index_margin(l[i]) + (l[j] - l[i] - 1);
                    feature[index] = 1.0;
                }
            }
        }
    }

    feature
}

#[cfg(test)]
mod tests {
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
            assert!((evaluator.evaluate(&cf, &next1, &next2) - 0.12496412).abs() <= 1e-3);
        }
    }
}
