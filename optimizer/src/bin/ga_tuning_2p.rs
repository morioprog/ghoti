use std::{
    collections::VecDeque,
    fs::create_dir_all,
    io::{stdout, Write},
    sync::{mpsc::channel, Arc, Mutex},
    time::Instant,
};

use chrono::Utc;
use clap::Parser;
use cpu::{
    bot::{BeamSearchAI, AI},
    evaluator::Evaluator,
};
use ghoti_optimizer::Mutateable;
use logger::{Logger, NullLogger};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use simulator::{haipuyo_detector::TUMO_PATTERN, simulate_2p, simulate_2p::SimulateResult2P};

#[derive(Parser)]
#[clap(
    name = "Ghoti GA Optimizer (2P)",
    author = "morioprog",
    version = "v0.0.1",
    about = "遺伝的アルゴリズムを用いたパラメータチューニング（2人対戦）"
)]
struct Opts {
    /// 何本先取させるか
    #[clap(long, default_value = "50")]
    win_goal: usize,

    /// AI に何手読みさせるか
    #[clap(long, default_value = "10")]
    visible_tumos: usize,

    /// 個体数
    #[clap(long, default_value = "20")]
    population_size: usize,

    /// 上位何個体はそのまま残すか
    #[clap(long, default_value = "5")]
    elite_size: usize,

    /// 何スレッドでシミュレーションするか
    #[clap(long, default_value = "12")]
    parallel: usize,

    /// ビームサーチの深さ
    #[clap(long, default_value = "10")]
    beam_depth: usize,

    /// ビームサーチの幅
    #[clap(long, default_value = "10")]
    beam_width: usize,

    /// ビームサーチにおけるモンテカルロ法の回数
    #[clap(long, default_value = "1")]
    beam_parallel: usize,
}

fn main() -> Result<(), std::io::Error> {
    let opts = Opts::parse();
    assert!(opts.elite_size < opts.population_size);

    let mut population = match std::fs::File::open("optimizer/logs/ga_tuning_2p/pop.json") {
        Ok(file) => serde_json::from_reader(file).unwrap_or_else(|e| {
            eprintln!("pop.json contained invalid data: {}", e);
            new_population(opts.population_size)
        }),
        Err(_) => new_population::<Evaluator>(opts.population_size),
    };

    // マルチスレッドでシミュレーション
    let matchups = Arc::new(Mutex::new((true, VecDeque::new())));
    let (send, game_results) = channel();
    for _ in 0..opts.parallel {
        let matchups = matchups.clone();
        let send = send.clone();
        std::thread::spawn(move || loop {
            // (1PのAIのindex, 1PのAIのEvaluator, 2PのAIのindex, 2PのAIのEvaluator, 配ぷよのマージン)
            let (p1, p1_e, p2, p2_e, haipuyo_margin) = {
                let (active, ref mut queue) = *matchups.lock().unwrap();
                if !active {
                    break;
                }
                match queue.pop_front() {
                    Some(v) => v,
                    None => continue,
                }
            };

            let mut logger: Box<dyn Logger> = Box::new(NullLogger::new("", None).unwrap());
            let ai_1p: Box<dyn AI> = Box::new(BeamSearchAI::new_customize(
                p1_e,
                opts.beam_depth,
                opts.beam_width,
                opts.beam_parallel,
            ));
            let ai_2p: Box<dyn AI> = Box::new(BeamSearchAI::new_customize(
                p2_e,
                opts.beam_depth,
                opts.beam_width,
                opts.beam_parallel,
            ));
            let simulate_result_2p = simulate_2p(
                &mut logger,
                &ai_1p,
                &ai_2p,
                opts.win_goal,
                opts.visible_tumos,
                Some(haipuyo_margin),
            );

            send.send(Some((p1, p2, simulate_result_2p))).ok();
        });
    }

    loop {
        assert_eq!(population.members.len(), opts.population_size);

        let start = Instant::now();

        // simulate_results[i * POPULATION_SIZE + j] := 個体 i と個体 j の対戦の棋譜 (i < j)
        let mut simulate_results: Vec<Option<SimulateResult2P>> = vec![];
        for _i in 0..opts.population_size {
            for _j in 0..opts.population_size {
                simulate_results.push(None);
            }
        }

        let haipuyo_margin = ((population.generation / 10) * 200) % TUMO_PATTERN;
        let mut count = 0;
        {
            let mut matchups = matchups.lock().unwrap();
            for i in 0..opts.population_size {
                for j in (i + 1)..opts.population_size {
                    matchups.1.push_back((
                        i,
                        population.members[i].clone(),
                        j,
                        population.members[j].clone(),
                        haipuyo_margin,
                    ));
                    count += 1;
                }
            }
        }

        println!(
            "> Gen {} (haipuyo-margin: {})",
            population.generation, haipuyo_margin
        );

        // results[i] := (個体番号, その個体が勝った回数)
        let mut results = vec![];
        for i in 0..opts.population_size {
            results.push((i, 0_i32));
        }
        for i in 0..count {
            if let Some((p1, p2, simulate_result_2p)) = game_results.recv().unwrap() {
                let simulate_result_2p = simulate_result_2p?;
                simulate_results[p1 * opts.population_size + p2] = Some(simulate_result_2p.clone());
                results[p1].1 += simulate_result_2p.win_count_1p as i32;
                results[p2].1 += simulate_result_2p.win_count_2p as i32;
            }
            print!("{} ", i + 1);
            stdout().flush().unwrap();
        }
        println!();

        results.sort_by_key(|(_, score)| -score);
        println!("> Gen {} Results:", population.generation);
        for i in 0..results.len() {
            let &(num, score) = &results[i];
            println!(
                "  {:>2}. {:<10}: {:>5} wins",
                i + 1,
                population.members[num].short_name(),
                score
            );
        }

        let weighted =
            rand::distributions::WeightedIndex::new(results.iter().map(|&(_, v)| v * v + 1))
                .unwrap();

        let mut new_population = Population {
            generation: population.generation + 1,
            members: vec![],
        };
        for &(i, _) in results.iter() {
            new_population.members.push(population.members[i].clone());
        }
        for i in opts.elite_size..population.members.len() {
            let p1 = thread_rng().sample(&weighted);
            let mut p2 = p1;
            while p1 == p2 {
                p2 = thread_rng().sample(&weighted);
            }
            new_population.members[i] = Evaluator::crossover(
                &population.members[p1],
                &population.members[p2],
                format!(
                    "Gen {} #{:>02}",
                    new_population.generation,
                    i - opts.elite_size
                ),
            );
        }

        // `pop.json` に今の世代のやつを記録
        serde_json::to_writer(
            std::fs::File::create("optimizer/logs/ga_tuning_2p/pop.json").unwrap(),
            &new_population,
        )
        .unwrap();

        match std::fs::File::create(format!(
            "optimizer/logs/ga_tuning_2p/best/pop_{}.json",
            population.generation
        )) {
            Ok(f) => serde_json::to_writer(std::io::BufWriter::new(f), &new_population.members[0])
                .unwrap_or_else(|e| eprintln!("Error saving best of generation: {}", e)),
            Err(e) => eprintln!("Error saving best of generation: {}", e),
        }

        // 上位2個体のindex (`best_ai_1` < `best_ai_2`)
        let best_ai_1 = results[0].0;
        let best_ai_2 = results[1].0;
        let (best_ai_1, best_ai_2) = (best_ai_1.min(best_ai_2), best_ai_1.max(best_ai_2));
        let best_ai_1_eval = population.members[best_ai_1].clone();
        let best_ai_2_eval = population.members[best_ai_2].clone();

        let file_dir = format!(
            "kifus/simulator_2p/ga_tuning_2p/{}_{}_vs_{}",
            population.generation,
            best_ai_1_eval.short_name(),
            best_ai_2_eval.short_name()
        );
        create_dir_all(&file_dir)?;

        let time_text = Utc::now().format("%Y%m%d_%H%M%S_%f");
        match std::fs::File::create(format!("{}/{}.json", &file_dir, &time_text)) {
            Ok(f) => serde_json::to_writer(
                std::io::BufWriter::new(f),
                &simulate_results[best_ai_1 * opts.population_size + best_ai_2]
                    .clone()
                    .unwrap(),
            )
            .unwrap_or_else(|e| eprintln!("Error saving best of generation: {}", e)),
            Err(e) => eprintln!("Error saving best kifu of generation: {}", e),
        }

        // Baselineと `opts.win_goal` 先してみる
        if population.members[results[0].0].sub_name.is_some() {
            let mut logger: Box<dyn Logger> = Box::new(NullLogger::new("", None).unwrap());
            let ai_strongest: Box<dyn AI> = Box::new(BeamSearchAI::new_customize(
                population.members[results[0].0].clone(),
                opts.beam_depth,
                opts.beam_width,
                opts.beam_parallel,
            ));
            let ai_baseline: Box<dyn AI> = Box::new(BeamSearchAI::new_customize(
                Evaluator::default(),
                opts.beam_depth,
                opts.beam_width,
                opts.beam_parallel,
            ));
            let simulate_result_with_baseline = simulate_2p(
                &mut logger,
                &ai_strongest,
                &ai_baseline,
                opts.win_goal,
                opts.visible_tumos,
                Some(0),
            )?;
            println!(
                "> {:>8} v.s. Baseline => {:3} - {:3}",
                population.members[results[0].0].short_name(),
                simulate_result_with_baseline.win_count_1p,
                simulate_result_with_baseline.win_count_2p
            );
        }

        let sec = start.elapsed().as_secs();
        let min = sec / 60;
        let sec = sec % 60;
        println!("> Elapsed: {:3}m {:>02}s", min, sec);
        println!();

        // 手動でこのファイルを作成するまでループする
        if std::fs::remove_file("optimizer/logs/ga_tuning_2p/end-request").is_ok() {
            break;
        }
        if std::fs::remove_file(format!(
            "optimizer/logs/ga_tuning_2p/end-request-{}",
            population.generation
        ))
        .is_ok()
        {
            break;
        }

        population = new_population;
    }

    // シミュレーション用のスレッドをcloseする
    matchups.lock().unwrap().0 = false;

    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Population<E: Mutateable> {
    generation: usize,
    members: Vec<E>,
}

fn new_population<E: Mutateable>(population_size: usize) -> Population<E> {
    let mut members = vec![];
    members.push(E::default());
    for num in 0..(population_size - 1) {
        members.push(E::generate(format!("Gen 0 #{:>02}", num)));
    }
    Population {
        generation: 0,
        members,
    }
}
