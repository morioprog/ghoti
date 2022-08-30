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
use simulator::{haipuyo_detector::TUMO_PATTERN, simulate_1p, simulate_1p::SimulateResult1P};

#[derive(Parser)]
#[clap(
    name = "Ghoti GA Optimizer (1P)",
    author = "morioprog",
    version = "v0.0.1",
    about = "遺伝的アルゴリズムを用いたパラメータチューニング（とこぷよ）"
)]
struct Opts {
    /// 最大手数
    #[clap(long, default_value = "50")]
    max_tumos: usize,

    /// AI に何手読みさせるか
    #[clap(long, default_value = "3")]
    visible_tumos: usize,

    /// 何回とこぷよさせるか
    #[clap(long, default_value = "10")]
    simulate_count: usize,

    /// この得点以上の連鎖が打たれたら終了
    #[clap(long, default_value = "10000")]
    required_chain_score: usize,

    /// 個体数
    #[clap(long, default_value = "20")]
    population_size: usize,

    /// 上位何個体はそのまま残すか
    #[clap(long, default_value = "5")]
    elite_size: usize,

    /// 何スレッドでシミュレーションするか
    #[clap(long, default_value = "1")]
    parallel: usize,
}

fn main() -> Result<(), std::io::Error> {
    let opts = Opts::parse();
    assert!(opts.elite_size < opts.population_size);

    let mut population = match std::fs::File::open("optimizer/logs/ga_tuning_1p/pop.json") {
        Ok(file) => serde_json::from_reader(file).unwrap_or_else(|e| {
            eprintln!("pop.json contained invalid data: {}", e);
            new_population(opts.population_size)
        }),
        Err(_) => new_population::<Evaluator>(opts.population_size),
    };

    // とこぷよのスコアを計算
    let calc_score = |sim_res: &SimulateResult1P| {
        ((sim_res.score as f64).powf(1.1f64)) as usize / sim_res.json_decisions.len()
    };

    // マルチスレッドでシミュレーション
    let matchups = Arc::new(Mutex::new((true, VecDeque::new())));
    let (send, game_results) = channel();
    for _ in 0..opts.parallel {
        let matchups = matchups.clone();
        let send = send.clone();
        std::thread::spawn(move || loop {
            // (AIのindex, AIのEvaluator, 何番の配ぷよから使うか)
            let (ai_index, ai_eval, haipuyo_margin) = {
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
            let ai: Box<dyn AI> = Box::new(BeamSearchAI::new_customize(ai_eval));

            let mut res = 0;
            let mut best: Option<SimulateResult1P> = None;
            for i in 0..opts.simulate_count {
                let simulate_result_1p = simulate_1p(
                    &mut logger,
                    &ai,
                    opts.visible_tumos,
                    opts.max_tumos,
                    Some((haipuyo_margin + i) % TUMO_PATTERN),
                    Some(opts.required_chain_score),
                )
                .unwrap();

                let score = calc_score(&simulate_result_1p);
                res += score;

                if let Some(sim) = best.clone() {
                    if score > calc_score(&sim) {
                        best = Some(simulate_result_1p);
                    }
                } else {
                    best = Some(simulate_result_1p);
                }
            }

            send.send(Some((ai_index, res / opts.simulate_count, best)))
                .ok();
        });
    }

    loop {
        assert_eq!(population.members.len(), opts.population_size);

        let start = Instant::now();

        // simulate_results[i] := 個体 i のとこぷよの結果
        let mut simulate_results: Vec<Option<SimulateResult1P>> = vec![None; opts.population_size];

        let haipuyo_margin =
            ((population.generation / opts.simulate_count) * opts.simulate_count) % TUMO_PATTERN;
        let mut count = 0;
        {
            let mut matchups = matchups.lock().unwrap();
            for i in 0..opts.population_size {
                matchups
                    .1
                    .push_back((i, population.members[i].clone(), haipuyo_margin));
                count += 1;
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
            if let Some((ai_index, res, sim_res)) = game_results.recv().unwrap() {
                results[ai_index].1 += res as i32;
                simulate_results[ai_index] = sim_res;
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
                "  {:>2}. {:<10}: {:>5} (best: {:>5})",
                i + 1,
                population.members[num].short_name(),
                score,
                calc_score(&simulate_results[num].clone().unwrap())
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
            std::fs::File::create("optimizer/logs/ga_tuning_1p/pop.json").unwrap(),
            &new_population,
        )?;

        match std::fs::File::create(format!(
            "optimizer/logs/ga_tuning_1p/best/pop_{}.json",
            population.generation
        )) {
            Ok(f) => serde_json::to_writer(std::io::BufWriter::new(f), &new_population.members[0])
                .unwrap_or_else(|e| eprintln!("Error saving best of generation: {}", e)),
            Err(e) => eprintln!("Error saving best of generation: {}", e),
        }

        let best_id = results[0].0;
        let best_eval = population.members[best_id].clone();
        let file_dir = format!(
            "kifus/simulator_1p/ga_tuning_1p/ga_{}",
            best_eval.short_name()
        );
        create_dir_all(&file_dir)?;

        let time_text = Utc::now().format("%Y%m%d_%H%M%S_%f");
        match std::fs::File::create(format!("{}/{}.json", &file_dir, &time_text)) {
            Ok(f) => serde_json::to_writer(
                std::io::BufWriter::new(f),
                &simulate_results[best_id].as_ref().unwrap(),
            )
            .unwrap_or_else(|e| eprintln!("Error saving best of generation: {}", e)),
            Err(e) => eprintln!("Error saving best kifu of generation: {}", e),
        }

        let sec = start.elapsed().as_secs();
        let min = sec / 60;
        let sec = sec % 60;
        println!("> Elapsed: {:3}m {:>02}s", min, sec);
        println!();

        // 手動でこのファイルを作成するまでループする
        if std::fs::remove_file("optimizer/logs/ga_tuning_1p/end-request").is_ok() {
            break;
        }
        if std::fs::remove_file(format!(
            "optimizer/logs/ga_tuning_1p/end-request-{}",
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
