use puyoai::{
    color::{Color, PuyoColor},
    field::{self, CoreField},
    kumipuyo::Kumipuyo,
};

/// 配ぷよを文字列に変換
pub fn convert_kumipuyo_seq(seq: &Vec<Kumipuyo>) -> Vec<String> {
    let mut tumos = vec![];
    for kumipuyo in seq {
        tumos.push(format!(
            "{}{}",
            kumipuyo.axis().as_str(),
            kumipuyo.child().as_str(),
        ));
    }
    tumos
}

/// 文字列を配ぷよに変換
pub fn revert_kumipuyo_seq(tumos: &Vec<String>) -> Vec<Kumipuyo> {
    let mut kumipuyos = vec![];
    for tumo in tumos {
        let bytes = tumo.as_bytes();
        let axis = PuyoColor::from_byte(bytes[0]);
        let child = PuyoColor::from_byte(bytes[1]);
        debug_assert!(axis.is_normal_color());
        debug_assert!(child.is_normal_color());
        kumipuyos.push(Kumipuyo::new(axis, child));
    }
    kumipuyos
}

/// 盤面を pfen-like な形式に変換
/// （pfen文字列: https://github.com/TukamotoRyuzo/upi-protocol/blob/master/README.md）
pub fn convert_core_field(cf: &CoreField) -> String {
    let mut ret = "".to_string();
    for x in 1..=field::WIDTH {
        for y in 1..=cf.height(x) {
            ret += match cf.color(x, y) {
                PuyoColor::RED => "r",
                PuyoColor::BLUE => "b",
                PuyoColor::YELLOW => "y",
                PuyoColor::GREEN => "g",
                PuyoColor::OJAMA => "o",
                _ => unreachable!(),
            };
        }
        ret += "/";
    }
    ret
}

/// pfen-like な文字列から盤面に変換
pub fn revert_core_field(pfen: &str) -> CoreField {
    let columns = pfen.split("/").collect::<Vec<&str>>();
    debug_assert_eq!(columns.len(), 7);

    let mut cf = CoreField::new();
    for x in 0..field::WIDTH {
        let bytes = columns[x].as_bytes();
        for y in 0..columns[x].len() {
            let color = PuyoColor::from_byte(bytes[y]);
            cf.field_mut().set_color(x + 1, y + 1, color);
        }
    }
    cf.update_height();
    cf
}

#[cfg(test)]
mod tests {
    use puyoai::color::PuyoColor;

    use super::*;

    #[test]
    fn test_convert_kumipuyo_seq() {
        assert_eq!(
            convert_kumipuyo_seq(&vec![Kumipuyo::new(PuyoColor::RED, PuyoColor::BLUE)]),
            vec!["RB"]
        );

        assert_eq!(
            convert_kumipuyo_seq(&vec![
                Kumipuyo::new(PuyoColor::YELLOW, PuyoColor::BLUE),
                Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
                Kumipuyo::new(PuyoColor::GREEN, PuyoColor::BLUE),
            ]),
            vec!["YB", "RR", "GB"]
        );
    }

    #[test]
    fn test_revert_kumipuyo_seq() {
        assert_eq!(
            revert_kumipuyo_seq(&vec!["RB".into()]),
            vec![Kumipuyo::new(PuyoColor::RED, PuyoColor::BLUE)],
        );

        assert_eq!(
            revert_kumipuyo_seq(&vec!["YB".into(), "RR".into(), "GB".into()]),
            vec![
                Kumipuyo::new(PuyoColor::YELLOW, PuyoColor::BLUE),
                Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
                Kumipuyo::new(PuyoColor::GREEN, PuyoColor::BLUE),
            ],
        );
    }

    #[test]
    fn test_convert_core_field() {
        assert_eq!(convert_core_field(&CoreField::new()), "//////");

        assert_eq!(
            convert_core_field(&CoreField::from_str(concat!(
                "G.....", // 10
                "Y.....", // 9
                "G....B", // 8
                "RR...B", // 7
                "GR...Y", // 6
                "GG.Y.Y", // 5
                "YYYBGG", // 4
                "GRBGYG", // 3
                "GGRBBY", // 2
                "RRBYYG"  // 1
            ))),
            "rggyggrgyg/rgrygrr/brby/ybgby/ybyg/gyggyybb/"
        );

        assert_eq!(
            convert_core_field(&CoreField::from_str(concat!(
                ".G.BRG", // 13
                "GBRRYR", // 12
                "RRYYBY", // 11
                "RGYRBR", // 10
                "YGYRBY", // 9
                "YGBGYR", // 8
                "GRBGYR", // 7
                "BRBYBY", // 6
                "RYYBYY", // 5
                "BRBYBR", // 4
                "BGBYRR", // 3
                "YGBGBG", // 2
                "RBGBGG"  // 1
            ))),
            "rybbrbgyyrrg/bggryrrgggrbg/gbbbybbbyyyr/bgyybyggrryrb/gbrbybyybbbyr/ggrryyrryryrg/"
        );
    }

    #[test]
    fn test_revert_core_field() {
        assert_eq!(revert_core_field("//////"), CoreField::new());

        assert_eq!(
            revert_core_field("rggyggrgyg/rgrygrr/brby/ybgby/ybyg/gyggyybb/"),
            CoreField::from_str(concat!(
                "G.....", // 10
                "Y.....", // 9
                "G....B", // 8
                "RR...B", // 7
                "GR...Y", // 6
                "GG.Y.Y", // 5
                "YYYBGG", // 4
                "GRBGYG", // 3
                "GGRBBY", // 2
                "RRBYYG"  // 1
            )),
        );

        assert_eq!(
            revert_core_field(concat!(
                "rybbrbgyyrrg/",
                "bggryrrgggrbg/",
                "gbbbybbbyyyr/",
                "bgyybyggrryrb/",
                "gbrbybyybbbyr/",
                "ggrryyrryryrg/"
            )),
            CoreField::from_str(concat!(
                ".G.BRG", // 13
                "GBRRYR", // 12
                "RRYYBY", // 11
                "RGYRBR", // 10
                "YGYRBY", // 9
                "YGBGYR", // 8
                "GRBGYR", // 7
                "BRBYBY", // 6
                "RYYBYY", // 5
                "BRBYBR", // 4
                "BGBYRR", // 3
                "YGBGBG", // 2
                "RBGBGG"  // 1
            )),
        );
    }
}
