use std::{collections::HashMap, fs::File, io::BufReader};

use serde::Deserialize;

struct OpeningMatcher {
    name: String,
    opener: Opener,
}

impl OpeningMatcher {
    fn new(json_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = format!("src/opening_matcher/jsons/{}", json_name);
        dbg!(path.clone());
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let opener: Opener = serde_json::from_reader(reader)?;

        Ok(Self {
            name: json_name.into(),
            opener,
        })
    }
}

#[derive(Debug, Deserialize)]
struct Opener {
    visible_tumos: usize,
    openings: HashMap<String, [usize; 2]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_json() {
        OpeningMatcher::new("sample.json").unwrap();
    }
}
