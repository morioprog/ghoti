// TODO: 他の特徴量も試しやすくする

use puyoai::{
    color::PuyoColor,
    field::{self, CoreField},
    kumipuyo::Kumipuyo,
};

pub const FEATURE_SIZE: usize = 3316;

pub fn convert_board_to_feature_indices(
    cf: &CoreField,
    next1: &Kumipuyo,
    next2: &Kumipuyo,
) -> Vec<usize> {
    #[inline]
    fn index_margin(idx: usize) -> usize {
        idx * (159 - idx) >> 1
    }
    #[inline]
    fn color_to_index(color: PuyoColor) -> usize {
        match color {
            PuyoColor::RED => 0,
            PuyoColor::BLUE => 1,
            PuyoColor::YELLOW => 2,
            PuyoColor::GREEN => 3,
            _ => unreachable!(),
        }
    }

    let mut feature_indices = vec![];

    let mut color_indices: [Vec<usize>; 4] = Default::default();
    color_indices[color_to_index(next1.axis())].push(0);
    color_indices[color_to_index(next1.child())].push(1);
    color_indices[color_to_index(next2.axis())].push(2);
    color_indices[color_to_index(next2.child())].push(3);

    for y in 1..=field::HEIGHT + 1 {
        let mut updated = false;
        for x in 1..=field::WIDTH {
            let index = (y - 1) * field::WIDTH + (x - 1);
            let color = cf.color(x, y);
            match color {
                PuyoColor::EMPTY => continue,
                PuyoColor::OJAMA => {
                    feature_indices.push(3160 + index);
                }
                PuyoColor::RED | PuyoColor::BLUE | PuyoColor::YELLOW | PuyoColor::GREEN => {
                    let color_index = color_to_index(color);
                    color_indices[color_index].push(index + 4);
                }
                _ => unreachable!(),
            }
            updated = true;
        }
        if !updated {
            break;
        }
    }

    for l in color_indices.iter() {
        if l.len() == 1 {
            feature_indices.push(3236 + l[0]);
        } else {
            for i in 0..l.len() {
                for j in i + 1..l.len() {
                    let index = index_margin(l[i]) + (l[j] - l[i] - 1);
                    feature_indices.push(index);
                }
            }
        }
    }

    feature_indices
}

pub fn convert_board_to_feature(
    cf: &CoreField,
    next1: &Kumipuyo,
    next2: &Kumipuyo,
) -> [f32; FEATURE_SIZE] {
    let mut feature = [0.0; FEATURE_SIZE];
    for i in convert_board_to_feature_indices(cf, next1, next2) {
        feature[i] = 1.0;
    }
    feature
}
