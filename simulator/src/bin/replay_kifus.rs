use std::fs;

use dialoguer::{theme::ColorfulTheme, Select};
use ghoti_simulator::{
    convert::{revert_core_field, revert_kumipuyo_seq},
    simulate_2p::{JsonEvent, SimulateResult2P},
};
use puyoai::{color::Color, field, kumipuyo::Kumipuyo};

macro_rules! show_prompt {
    ($selections:ident, $message:literal) => {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt($message)
            .default(0)
            .items(&$selections[..])
            .interact()?
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let selections = &["simulator_1p: Tokopuyo", "simulator_2p: Battle"];
    let kifu_type = show_prompt!(selections, "Pick kifu type");
    let kifu_dir = match kifu_type {
        0 => "./kifus/simulator_1p",
        1 => "./kifus/simulator_2p",
        _ => unreachable!(),
    };

    let selections = list_file_names(&kifu_dir)?;
    let kifu_name = show_prompt!(selections, "Pick kifu name");
    let kifu_name = selections[kifu_name].clone();
    let kifu_dir = format!("{}/{}", kifu_dir, kifu_name);

    if kifu_name.contains("ga_tuning") {
        todo!()
    }

    let selections = list_file_names(&kifu_dir)?;
    let kifu_json = show_prompt!(selections, "Pick kifu json");
    let kifu_json = selections[kifu_json].clone();
    let kifu_path = format!("{}/{}", kifu_dir, kifu_json);

    match kifu_type {
        0 => todo!(),
        1 => show_2p_kifu(&kifu_path),
        _ => unreachable!(),
    }?;

    Ok(())
}

fn list_file_names(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(fs::read_dir(dir)?
        .map(|entry| {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let file_name = entry_path.file_name().unwrap();
            let file_name_as_str = file_name.to_str().unwrap();
            String::from(file_name_as_str)
        })
        .collect())
}

fn show_2p_kifu(json_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(json_path)?;
    let simulate_result: SimulateResult2P = serde_json::from_reader(file)?;
    let json_matches = &simulate_result.json_matches;
    let selections = json_matches
        .iter()
        .map(|json_match| {
            let last_json_event = json_match.json_events.iter().last().unwrap();
            format!(
                "[{}] {:6} - {:6}",
                if json_match.won_1p { "1P" } else { "2P" },
                last_json_event.json_state_1p.score,
                last_json_event.json_state_2p.score
            )
        })
        .collect::<Vec<String>>();
    let json_match = show_prompt!(selections, "Pick match");
    let json_match = &json_matches[json_match];

    let haipuyo = revert_kumipuyo_seq(&json_match.tumos);
    for json_event in &json_match.json_events {
        show_json_event(&haipuyo, &json_event);
    }

    Ok(())
}

fn show_json_event(haipuyo: &Vec<Kumipuyo>, json_event: &JsonEvent) {
    println!("> {:5} F", json_event.frame);

    let json_state_1p = &json_event.json_state_1p;
    let json_state_2p = &json_event.json_state_2p;

    let cf_1p = revert_core_field(&json_state_1p.field);
    let cf_2p = revert_core_field(&json_state_2p.field);

    let tumo_index_1p = json_state_1p.tumo_index;
    let tumo_index_2p = json_state_2p.tumo_index;

    let tumo_index_1p_next = (tumo_index_1p + 1) % haipuyo.len();
    let tumo_index_2p_next = (tumo_index_2p + 1) % haipuyo.len();

    // おじゃまぷよ
    println!(
        "   {:4} ({:4})             {:4} ({:4})",
        json_state_1p.ojama_fixed,
        json_state_1p.ojama_ongoing,
        json_state_2p.ojama_fixed,
        json_state_2p.ojama_ongoing
    );

    for y in (0..(field::MAP_HEIGHT)).rev() {
        for x in 0..(field::MAP_WIDTH) {
            print!("{}", cf_1p.color(x, y).as_colored_str_wide());
        }

        match field::MAP_HEIGHT - y {
            3 => print!("{}", haipuyo[tumo_index_1p].child().as_colored_str_wide()),
            4 => print!("{}", haipuyo[tumo_index_1p].axis().as_colored_str_wide()),
            6 => print!(
                "{}",
                haipuyo[tumo_index_1p_next].child().as_colored_str_wide()
            ),
            7 => print!(
                "{}",
                haipuyo[tumo_index_1p_next].axis().as_colored_str_wide()
            ),
            _ => print!("  "),
        }

        print!("    ");

        match field::MAP_HEIGHT - y {
            3 => print!("{}", haipuyo[tumo_index_2p].child().as_colored_str_wide()),
            4 => print!("{}", haipuyo[tumo_index_2p].axis().as_colored_str_wide()),
            6 => print!(
                "{}",
                haipuyo[tumo_index_2p_next].child().as_colored_str_wide()
            ),
            7 => print!(
                "{}",
                haipuyo[tumo_index_2p_next].axis().as_colored_str_wide()
            ),
            _ => print!("  "),
        }

        for x in 0..(field::MAP_WIDTH) {
            print!("{}", cf_2p.color(x, y).as_colored_str_wide());
        }
        println!();
    }

    // 点数
    println!(
        "  #{:03} {:07}            #{:03} {:07}",
        tumo_index_1p, json_state_1p.score, tumo_index_2p, json_state_2p.score
    );

    println!();
}
