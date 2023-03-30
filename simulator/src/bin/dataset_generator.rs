// TODO: simulate_1p と被りまくってるので共通化したい

use std::{
    fs::File,
    io::{self, Write},
    sync::{Arc, Mutex},
    thread,
};

use cpu::evaluator::pattern_matching::Evaluator;
use ghoti_simulator::{convert::convert_core_field, haipuyo_detector::HaipuyoDetector};
use indicatif::{MultiProgress, ProgressBar, ProgressIterator, ProgressStyle};
use puyoai::{
    color::Color, decision::Decision, field::CoreField, kumipuyo::Kumipuyo, plan::Plan,
    rensa_result::RensaResult,
};
use rand::Rng;

// TODO: これはコマンドライン引数で指定できるようにしたい
const WIDE_BEAM_WIDTH: usize = 10_000;

// 広いビームサーチをして初期盤面を生成する
// （上位ではなく全体から均等に取り出すのでビームサーチではない）
fn main() -> io::Result<()> {
    // TODO: 出力先を吟味する
    let file = File::create("pretrain.txt")?;
    let file = Arc::new(Mutex::new(file));

    let max_depth = 100;
    let state_per_depth = 400;

    let mp = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
    )
    .unwrap();

    let mut handles = vec![];

    let parallel = 24;
    for _ in 0..parallel {
        let file = Arc::clone(&file);
        let pb = mp.add(ProgressBar::new(max_depth as u64));
        pb.set_style(sty.clone());
        let handle = thread::spawn(move || {
            generate_initial_states(max_depth, state_per_depth, file, pb);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

// TODO: たまにおじゃまを降らせる
fn generate_initial_states(
    max_depth: usize,
    state_per_depth: usize,
    file: Arc<Mutex<File>>,
    pb: ProgressBar,
) {
    let evaluator = Evaluator {
        // 与えられた盤面の情報のみで評価を行う
        chain: 0,
        chain_sq: 0,
        chain_score: 0,
        chain_frame: 0,
        chigiri: 0,
        move_frame: 0,
        ..Evaluator::default()
    };
    let cf = CoreField::new();
    // TODO: ツモパターンが被るかもしれない
    let seq = HaipuyoDetector::random_haipuyo();

    let mut state_v: Vec<State> = vec![State::new(&evaluator, &cf, &seq[0], &seq[1])];
    {
        let mut file = file.lock().unwrap();
        writeln!(file, "{}", state_v[0].to_str()).unwrap();
    }

    let mut max_score = i32::min_value();
    let mut min_score = i32::max_value();

    for depth in (0..max_depth).progress_with(pb) {
        let mut next_state_v: Vec<State> =
            Vec::with_capacity(WIDE_BEAM_WIDTH * Decision::all_valid_decisions().len());
        for cur_state in &state_v {
            enumerate_next_states(&cur_state, &mut next_state_v, &seq[depth + 2], &evaluator);
        }

        next_state_v.sort_by(|a, b| b.score.cmp(&a.score));
        max_score = max_score.max(next_state_v.first().unwrap().score);
        min_score = min_score.min(next_state_v.last().unwrap().score);

        state_v.clear();
        sample_states(&next_state_v, &mut state_v, state_per_depth);
        {
            let mut file = file.lock().unwrap();
            for state in &state_v {
                writeln!(file, "{}", state.to_str()).unwrap();
            }
        }

        state_v.clear();
        sample_states(&next_state_v, &mut state_v, WIDE_BEAM_WIDTH);
    }
}

fn enumerate_next_states(
    cur_state: &State,
    next_state_v: &mut Vec<State>,
    next_next2: &Kumipuyo,
    evaluator: &Evaluator,
) {
    let seq = vec![cur_state.next1.clone()];

    Plan::iterate_available_plans(&cur_state.field, &seq, 1, &mut |plan: &Plan| {
        next_state_v.push(State::new(
            evaluator,
            &plan.field(),
            &cur_state.next2,
            &next_next2,
        ));
    });
}

fn sample_states(state_v: &Vec<State>, next_state_v: &mut Vec<State>, sample_len: usize) {
    if state_v.len() <= sample_len {
        next_state_v.extend_from_slice(state_v);
        return;
    }

    let mut rng = rand::thread_rng();
    let bucket = state_v.len() / sample_len;
    for i in 0..sample_len {
        let idx = rng.gen_range((bucket * i)..(bucket * (i + 1)));
        next_state_v.push(state_v[idx].clone());
    }
}

#[derive(Clone)]
struct State {
    field: CoreField,
    next1: Kumipuyo,
    next2: Kumipuyo,
    score: i32,
}

impl State {
    fn new(evaluator: &Evaluator, cf: &CoreField, next1: &Kumipuyo, next2: &Kumipuyo) -> State {
        let plan = Plan::new(
            cf.clone(),
            vec![],
            RensaResult::empty(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            false,
        );
        let score = evaluator.evaluate(&plan);

        State {
            field: cf.clone(),
            next1: next1.clone(),
            next2: next2.clone(),
            score,
        }
    }

    fn to_str(&self) -> String {
        format!(
            "{},{}{},{}{},{}",
            convert_core_field(&self.field),
            self.next1.axis().to_char().to_lowercase(),
            self.next1.child().to_char().to_lowercase(),
            self.next2.axis().to_char().to_lowercase(),
            self.next2.child().to_char().to_lowercase(),
            self.score // TODO: [0, 1] の範囲に収める
        )
    }
}
