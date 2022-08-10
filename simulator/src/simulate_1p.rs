use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
};

use chrono::{DateTime, Utc};
use cpu::bot::*;
use logger::Logger;
use puyoai::{
    color::Color, decision::Decision, field::CoreField, kumipuyo::Kumipuyo, puyop::make_puyop_url,
    serde_def::DecisionDef,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::{convert::convert_kumipuyo_seq, haipuyo_detector::*};

pub fn simulate_1p(
    logger: &mut Box<dyn Logger>,
    ai: &Box<dyn AI>,
    visible_tumos: usize,
    max_tumos: usize,
    haipuyo_margin: Option<usize>, // Noneならランダムに、Someならその番号の配ぷよを使う
    required_chain_score: Option<usize>, // この得点以上の連鎖が打たれたら終了
) -> Result<SimulateResult1P, std::io::Error> {
    logger.print(format!("> AI: {} ({:3}手読み)\n", ai.name(), visible_tumos))?;

    // TODO: フレームを更新する
    let seq = match haipuyo_margin {
        None => HaipuyoDetector::random_haipuyo(),
        Some(margin) => HaipuyoDetector::retrieve_haipuyo(margin % TUMO_PATTERN),
    };
    let mut player_state = PlayerState::initial_state(vec![], Some(seq.clone()));

    let mut ai_decisions: Vec<AIDecision> = vec![];
    let mut decisions: Vec<Decision> = vec![];
    let mut score = 0;

    for tumo_index in 1..=max_tumos {
        // AI に考えさせる
        player_state.set_seq(visible_tumos);
        let ai_decision = ai.think(&player_state, None);
        ai_decisions.push(ai_decision.clone());
        decisions.push(ai_decision.decisions[0].clone());

        // 実際にぷよを落とす
        player_state.drop_kumipuyo(&ai_decision.decisions[0]);
        let rensa_result = player_state.field.simulate();
        score += rensa_result.score;

        logger.print(format!(
            "{:3}. {}{} ({}, {}) [{:4} ms] {:7} (+{:6}) | {}\n",
            tumo_index,
            player_state.seq[0].axis().as_str(),
            player_state.seq[0].child().as_str(),
            ai_decision.decisions[0].axis_x(),
            ai_decision.decisions[0].rot(),
            ai_decision.elapsed.as_millis(),
            score,
            rensa_result.score,
            ai_decision.log_output
        ))?;

        // `required_chain_score` の得点以上の連鎖が打たれた
        if let Some(score) = required_chain_score && score <= rensa_result.score {
            break;
        }

        // 死んだ
        if player_state.field.is_dead() {
            break;
        }

        player_state.tumo_index += 1;
    }

    let ret = SimulateResult1P::new(score, visible_tumos, &seq, &decisions, &ai_decisions);
    logger.print(ret.url.clone())?;

    Ok(ret)
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct JsonDecision {
    think_ms: u128,
    log_output: String,
    #[serde_as(as = "Vec<DecisionDef>")]
    decisions: Vec<Decision>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulateResult1P {
    date: DateTime<Utc>,
    pub score: usize,
    visible_tumos: usize,
    tumos: Vec<String>, // ["RR", "YG", ...]
    pub json_decisions: Vec<JsonDecision>,
    url: String,
}

impl SimulateResult1P {
    fn new(
        score: usize,
        visible_tumos: usize,
        seq: &Vec<Kumipuyo>,
        decisions: &Vec<Decision>,
        ai_decisions: &Vec<AIDecision>,
    ) -> Self {
        let url = make_puyop_url(&CoreField::new(), seq, decisions);
        let tumos = convert_kumipuyo_seq(&seq);
        let json_decisions = {
            let mut json_decisions = vec![];
            for ai_decision in ai_decisions {
                json_decisions.push(JsonDecision {
                    think_ms: ai_decision.elapsed.as_millis(),
                    log_output: ai_decision.log_output.clone(),
                    decisions: ai_decision.decisions.clone(),
                });
            }
            json_decisions
        };

        SimulateResult1P {
            date: Utc::now(),
            score,
            visible_tumos,
            tumos,
            json_decisions,
            url,
        }
    }

    pub fn export_json(&self, pr_number: usize, ai_name: &str) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_string(&self).unwrap();

        let file_dir = format!("kifus/simulator_1p/{}_{}", pr_number, &ai_name);
        create_dir_all(&file_dir)?;

        let time_text = Utc::now().format("%Y%m%d_%H%M%S_%f");
        let file_path = format!("{}/{}.json", &file_dir, &time_text);
        let mut buf_writer = BufWriter::new(File::create(&file_path)?);

        write!(buf_writer, "{}", &serialized)?;
        buf_writer.flush()?;

        Ok(())
    }
}
