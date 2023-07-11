#![feature(test)]
extern crate test;
use ghoti_cpu::evaluator::nnue::{
    naive_evaluator::NaiveEvaluator, nalgebra_evaluator::NalgebraEvaluator,
    ndarray_evaluator::NdarrayEvaluator,
};
use puyoai::{color::PuyoColor, field::CoreField, kumipuyo::Kumipuyo};

// 33,160 ns/iter (+/- 1,190)
// 33,194 ns/iter (+/- 1,707)
// 33,984 ns/iter (+/- 8,563)
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

// evaluate_slow
// 5,904,164 ns/iter (+/- 2,352,125)
// 5,817,197 ns/iter (+/- 2,751,774)
// 5,621,517 ns/iter (+/- 5,502,120)
// evaluate (128x32)
// 12,918 ns/iter (+/- 7,685)
// 13,123 ns/iter (+/- 2,948)
// 12,874 ns/iter (+/- 5,502)
// evaluate (32x32)
// 7,802 ns/iter (+/- 270)
// 7,864 ns/iter (+/- 353)
// 7,806 ns/iter (+/- 1,354)
#[bench]
fn bench_ndarray_evaluator(b: &mut test::Bencher) {
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

    let evaluator = NdarrayEvaluator::new();
    b.iter(|| test::black_box(evaluator.evaluate(&cf, &next1, &next2)));
}

// 32x32
// 2,662 ns/iter (+/- 137)
// 2,748 ns/iter (+/- 146)
// 2,640 ns/iter (+/- 91)
#[bench]
fn bench_nalgebra_evaluator(b: &mut test::Bencher) {
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

    let evaluator = NalgebraEvaluator::new();
    b.iter(|| test::black_box(evaluator.evaluate(&cf, &next1, &next2)));
}
