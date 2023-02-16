#![feature(test)]
extern crate test;
use ghoti_cpu::evaluator::Evaluator;
use puyoai::{decision::Decision, field::CoreField, plan::Plan};

#[bench]
fn bench_evaluator(b: &mut test::Bencher) {
    let mut cf = CoreField::from_str(concat!(
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
    let decisions = vec![Decision::new(2, 2), Decision::new(1, 1)];
    let rensa_result = cf.simulate();
    let plan = Plan::new(
        cf,
        decisions,
        rensa_result,
        1,
        342,
        1200,
        10,
        20,
        30,
        40,
        false,
    );

    let evaluator = Evaluator::default();
    b.iter(|| test::black_box(evaluator.clone().evaluate(&plan.clone())));
}
