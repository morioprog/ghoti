//! 配ぷよを序盤8手の順番にソートしているため、
//! 順番に `retrieve_haipuyo` を呼ぶと序盤の手が被る点に注意

use puyoai::{
    color::{Color, PuyoColor},
    kumipuyo::Kumipuyo,
};
use rand::Rng;

use super::haipuyo;

pub const TUMO_PATTERN: usize = 65536;

pub struct HaipuyoDetector {}

impl HaipuyoDetector {
    pub fn hash_head_8(head_8: &Vec<Kumipuyo>) -> u32 {
        debug_assert_eq!(8, head_8.len());

        let mut hash: u32 = 0;
        for tumo in head_8 {
            hash <<= 2;
            hash += tumo.axis() as u32 - 4;
            hash <<= 2;
            hash += tumo.child() as u32 - 4;
        }
        hash
    }

    pub fn search_key(x: u32) -> usize {
        let mut ok: usize = 0;
        let mut ng: usize = TUMO_PATTERN;

        while ng - ok > 1 {
            let md = (ok + ng) >> 1;
            if haipuyo::HAIPUYO_KEYS[md] <= x {
                ok = md;
            } else {
                ng = md;
            }
        }

        // TODO: assert
        // assert_eq!(x, haipuyo::HAIPUYO_KEYS[ok]);

        ok
    }

    pub fn u64_to_seq(y: u64) -> Vec<Kumipuyo> {
        let mut seq: Vec<Kumipuyo> = vec![];
        let mut x = y;
        for _ in 0..16 {
            let axis = PuyoColor::from_u32(((x & 3) + 4) as u32);
            x >>= 2;
            let child = PuyoColor::from_u32(((x & 3) + 4) as u32);
            x >>= 2;
            seq.push(Kumipuyo::new(axis, child));
        }
        seq
    }

    pub fn retrieve_haipuyo(key: usize) -> Vec<Kumipuyo> {
        debug_assert!(key < TUMO_PATTERN);

        let mut seq: Vec<Kumipuyo> = vec![];
        seq.append(&mut HaipuyoDetector::u64_to_seq(haipuyo::HAIPUYO_0[key]));
        seq.append(&mut HaipuyoDetector::u64_to_seq(haipuyo::HAIPUYO_1[key]));
        seq.append(&mut HaipuyoDetector::u64_to_seq(haipuyo::HAIPUYO_2[key]));
        seq.append(&mut HaipuyoDetector::u64_to_seq(haipuyo::HAIPUYO_3[key]));
        seq.append(&mut HaipuyoDetector::u64_to_seq(haipuyo::HAIPUYO_4[key]));
        seq.append(&mut HaipuyoDetector::u64_to_seq(haipuyo::HAIPUYO_5[key]));
        seq.append(&mut HaipuyoDetector::u64_to_seq(haipuyo::HAIPUYO_6[key]));
        seq.append(&mut HaipuyoDetector::u64_to_seq(haipuyo::HAIPUYO_7[key]));
        seq
    }

    pub fn search_haipuyo(head_8: &Vec<Kumipuyo>) -> Vec<Kumipuyo> {
        let hash: u32 = HaipuyoDetector::hash_head_8(head_8);
        let key: usize = HaipuyoDetector::search_key(hash);

        // TODO: assert
        if haipuyo::HAIPUYO_KEYS[key] != hash {
            return vec![];
        }

        HaipuyoDetector::retrieve_haipuyo(key)
    }

    pub fn random_haipuyo() -> Vec<Kumipuyo> {
        let key = rand::thread_rng().gen_range(0..TUMO_PATTERN);

        HaipuyoDetector::retrieve_haipuyo(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ゲーム内の色（'rgbyp')が、'rbyg'に対応していることに注意する
    #[test]
    fn test_search_haipuyo() {
        // 24858 番目の配ぷよ
        let head = vec![
            Kumipuyo::new(PuyoColor::BLUE, PuyoColor::GREEN),
            Kumipuyo::new(PuyoColor::GREEN, PuyoColor::GREEN),
            Kumipuyo::new(PuyoColor::GREEN, PuyoColor::YELLOW),
            Kumipuyo::new(PuyoColor::BLUE, PuyoColor::BLUE),
            Kumipuyo::new(PuyoColor::BLUE, PuyoColor::BLUE),
            Kumipuyo::new(PuyoColor::RED, PuyoColor::BLUE),
            Kumipuyo::new(PuyoColor::RED, PuyoColor::YELLOW),
            Kumipuyo::new(PuyoColor::BLUE, PuyoColor::YELLOW),
            Kumipuyo::new(PuyoColor::RED, PuyoColor::RED),
            Kumipuyo::new(PuyoColor::BLUE, PuyoColor::YELLOW),
            Kumipuyo::new(PuyoColor::GREEN, PuyoColor::BLUE),
            Kumipuyo::new(PuyoColor::YELLOW, PuyoColor::GREEN),
            Kumipuyo::new(PuyoColor::RED, PuyoColor::BLUE),
            Kumipuyo::new(PuyoColor::YELLOW, PuyoColor::YELLOW),
        ];

        let haipuyo = HaipuyoDetector::search_haipuyo(&head.get(0..8).unwrap().to_vec());
        assert_eq!(haipuyo.len(), 128);
        for i in 0..head.len() {
            assert_eq!(haipuyo[i], head[i]);
        }
    }
}
