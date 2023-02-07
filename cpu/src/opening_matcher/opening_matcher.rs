use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
};

use itertools::Itertools;
use puyoai::{color::Color, decision::Decision, field, field::CoreField, kumipuyo::Kumipuyo};
use serde::Deserialize;

pub struct OpeningMatcher {
    _name: String,
    opener: Opener,
}

impl OpeningMatcher {
    pub fn new(json_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = format!(
            "{}/src/opening_matcher/jsons/{}",
            env!("CARGO_MANIFEST_DIR"),
            json_name
        );
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let opener: Opener = serde_json::from_reader(reader)?;

        Ok(Self {
            _name: json_name.into(),
            opener,
        })
    }

    pub fn find_opening(
        &self,
        tumo_index: usize,
        field: &CoreField,
        seq: &Vec<Kumipuyo>,
    ) -> Option<Decision> {
        let appeared_colors = appeared_colors(field, seq);
        let n = appeared_colors.len();
        let letters = (0..n).map(|i| (b'A' + i as u8) as char);

        let opener = &self.opener.openings[tumo_index];
        for perm in letters.permutations(n) {
            let mut map: HashMap<char, char> = HashMap::new();
            for (color, letter) in appeared_colors.iter().zip(perm.iter()) {
                map.insert(*color, *letter);
            }

            // TODO: rev が不安定（appeared_colors に依存する）
            let (tumos_s, rev) = tumos_to_abcd(&map, seq);
            let field_s = field_to_abcd(&map, field);

            if !opener.contains_key(&tumos_s) {
                continue;
            }
            let opener = opener.get(&tumos_s).unwrap();
            if !opener.contains_key(&field_s) {
                continue;
            }

            let decision = opener.get(&field_s).unwrap();
            let decision = if rev {
                match decision[1] {
                    0 => Decision::new(decision[0], 2),
                    1 => Decision::new(decision[0] + 1, 3),
                    2 => Decision::new(decision[0], 0),
                    3 => Decision::new(decision[0] - 1, 1),
                    _ => {
                        println!("invalid decision!");
                        Decision::new(decision[0], decision[1])
                    }
                }
            } else {
                Decision::new(decision[0], decision[1])
            };
            return Some(decision);
        }

        None
    }
}

// TODO: HashSet<PuyoColor> を返したい
fn appeared_colors(field: &CoreField, seq: &Vec<Kumipuyo>) -> HashSet<char> {
    let mut appeared_colors = HashSet::new();

    // seq に含まれる PuyoColor を列挙
    for kumipuyo in seq.iter() {
        appeared_colors.insert(kumipuyo.axis().to_char());
        appeared_colors.insert(kumipuyo.child().to_char());
    }

    // field に含まれる PuyoColor を列挙
    for x in 1..field::WIDTH + 1 {
        for y in 1..field::HEIGHT + 1 {
            let color = field.color(x, y);
            if color.is_normal_color() {
                appeared_colors.insert(color.to_char());
            }
        }
    }

    appeared_colors
}

// (ABCD で表したツモ, 初手を反転したか否か)
fn tumos_to_abcd(map: &HashMap<char, char>, seq: &Vec<Kumipuyo>) -> (String, bool) {
    let mut s = String::new();
    let mut rev = None;
    for kumipuyo in seq.iter() {
        let axis = kumipuyo.axis().to_char();
        let child = kumipuyo.child().to_char();
        let axis = *map.get(&axis).unwrap();
        let child = *map.get(&child).unwrap();
        let (axis, child, reved) = if axis <= child {
            (axis, child, false)
        } else {
            (child, axis, true)
        };

        s.push(axis);
        s.push(child);

        if rev.is_none() {
            rev = Some(reved);
        }
    }
    (s, rev.unwrap())
}

// ABCD で表した盤面
fn field_to_abcd(map: &HashMap<char, char>, field: &CoreField) -> String {
    let mut s = String::new();
    for x in 1..field::WIDTH + 1 {
        for y in 1..field::HEIGHT + 1 {
            let color = field.color(x, y);
            if color.is_normal_color() {
                s.push(*map.get(&color.to_char()).unwrap());
            }
        }
        s.push('/');
    }
    s
}

#[derive(Debug, Deserialize)]
struct Opener {
    #[serde(rename = "visible_tumos")]
    _visible_tumos: usize,
    // TODO: serde_def::DecisionDef を使う
    openings: [HashMap<String, HashMap<String, [usize; 2]>>; 8],
}

#[cfg(test)]
mod tests {
    use puyoai::color::PuyoColor;

    use super::*;

    #[test]
    fn test_sample_json() {
        OpeningMatcher::new("sample.json").unwrap();
    }

    #[test]
    fn test_find_opening() {
        let openings = OpeningMatcher::new("sample.json").unwrap();
        assert_eq!(
            openings.find_opening(
                0,
                &CoreField::new(),
                &vec![
                    Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
                    Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
                ]
            ),
            Some(Decision::new(3, 2)),
        );
        // FIXME: (6, 0) になる場合がある
        // assert_eq!(
        //     openings.find_opening(
        //         3,
        //         &CoreField::from_str(concat!(
        //             "..G...", // 2
        //             "GGBGG.", // 1
        //         )),
        //         &vec![
        //             Kumipuyo::new(PuyoColor::YELLOW, PuyoColor::RED),
        //             Kumipuyo::new(PuyoColor::GREEN, PuyoColor::BLUE),
        //         ]
        //     ),
        //     Some(Decision::new(6, 2)),
        // );
    }
}
