#![feature(test)]
extern crate test;
use ghoti_cpu::evaluator::nnue::naive_evaluator::NaiveEvaluator;
use puyoai::{color::PuyoColor, field::CoreField, kumipuyo::Kumipuyo};

#[bench]
fn bench_naive_onnx_evaluator(b: &mut test::Bencher) {
    let cf = CoreField::from_str(concat!(
        ".GY..G", // 11
        ".GGYBG", // 10
        "YGRYYB", // 9
        "BRGGGB", // 8
        "BYRRYB", // 7
        "YYRYGY", // 6
        "BBBYGG", // 5
        "YYYGRR", // 4
        "GRBRYR", // 3
        "GGRBBB", // 2
        "RRBYYY"  // 1
    ));
    let next1 = Kumipuyo::new(PuyoColor::GREEN, PuyoColor::RED);
    let next2 = Kumipuyo::new(PuyoColor::RED, PuyoColor::BLUE);

    let onnx_name = "data=660000-lr=1e-4";
    let mut evaluator = NaiveEvaluator::new(onnx_name).unwrap();
    b.iter(|| test::black_box(evaluator.evaluate(&cf, &next1, &next2)));
}
