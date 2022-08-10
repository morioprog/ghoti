use puyoai::{
    color::PuyoColor,
    field::{self, CoreField},
};

macro_rules! detect_shape {
    (
        $name:ident
        $([$($color:tt)*])*
    ) => {
        pub fn $name(cf: &CoreField) -> bool {
            // corr: `A` などと実際の色との対応関係
            let mut corr = [PuyoColor::EMPTY; 5];
            // 直前（上）のトークン
            let mut prv_row = [detect_shape!(@color _); field::WIDTH];
            // 縦方向のインデックス
            let y = detect_shape!(@len [$([$($color)*])*]);

            $(
                {
                    // 直前（左）のトークン
                    let prv = detect_shape!(@color _);
                    // 横方向のインデックス
                    let x = 1;

                    $(
                        // マクロの引数で渡したやつ
                        let color = detect_shape!(@color $color);
                        // `_`（なんでもOK）じゃなかったら...
                        if color != detect_shape!(@color _) {
                            if corr[color] == PuyoColor::EMPTY {
                                // まだその色が使われてない
                                corr[color] = cf.color(x, y);
                            } else if corr[color] != cf.color(x, y) {
                                // 色の対応関係が合っていない
                                return false;
                            }
                        }
                        if x > 1 && prv != color && cf.color(x - 1, y) == cf.color(x, y) {
                            // トークンが違うのに左右で同じ色
                            return false;
                        }
                        if prv_row[x - 1] != color && cf.color(x, y + 1) == cf.color(x, y) {
                            // トークンが違うのに上下で同じ色
                            return false;
                        }

                        prv_row[x - 1] = color;
                        #[allow(unused)]
                        let prv = color;
                        #[allow(unused)]
                        let x = x + 1;
                    )*
                }
                #[allow(unused)]
                let y = y - 1;
            )*
            true
        }
    };
    // NOTE: puyoaiでは3色までだった
    (@color A) => { 0 };
    (@color B) => { 1 };
    (@color C) => { 2 };
    (@color D) => { 3 };
    (@color E) => { 4 };
    (@color _) => { 9 };
    // 配列の長さ
    (@len []) => { 0 };
    (@len [$_:tt $($rest:tt)*]) => { 1 + detect_shape!(@len [$($rest)*]) }
}

// GTR
detect_shape! {
    gtr_base_1
    [A A _ _ _ _]
}
detect_shape! {
    gtr_base_2
    [B B _ _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_base_3
    [B _ _ _ _ _]
    [B B _ _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_base_4
    [B B A _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_base_5
    [B _ _ _ _ _]
    [A _ _ _ _ _]
}
detect_shape! {
    gtr_base_6
    [B _ _ _ _ _]
    [A A C _ _ _]
}
detect_shape! {
    gtr_base_7
    [_ _ A _ _ _]
    [A A B _ _ _]
}
detect_shape! {
    gtr_base_8
    [B A _ _ _ _]
    [B B _ _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_1
    [C A B _ _ _]
    [C C A B _ _]
    [A A B B _ _]
}
detect_shape! {
    gtr_2
    [C A B _ _ _]
    [C C A B B _]
    [A A B _ _ _]
}
detect_shape! {
    gtr_3
    [C A B _ _ _]
    [C C A B B B]
    [A A B _ _ _]
}
detect_shape! {
    gtr_4
    [C A B _ _ _]
    [C C A B B _]
    [A A B _ B _]
}
detect_shape! {
    gtr_5
    [C A B _ _ _]
    [C C A B B _]
    [A A _ B _ _]
}
detect_shape! {
    gtr_6
    [C A B _ _ _]
    [C C A B B B]
    [A A _ _ _ _]
}

// 連鎖尾側
detect_shape! {
    gtr_tail_1_1
    [_ A B C _ _]
    [_ _ A B C C]
    [A A B B C _]
}
detect_shape! {
    gtr_tail_1_2
    [_ A B C _ _]
    [_ _ A B C _]
    [A A B B C C]
}
detect_shape! {
    gtr_tail_1_3
    [_ _ _ C _ _]
    [_ A B C _ _]
    [_ _ A B C _]
    [A A B B C _]
}
detect_shape! {
    gtr_tail_2_1
    [_ A B _ C _]
    [_ _ A B B C]
    [A A B C C _]
}
detect_shape! {
    gtr_tail_2_2
    [_ A B _ C _]
    [_ _ A B B _]
    [A A B C C C]
}
detect_shape! {
    gtr_tail_2_3
    [_ A B _ C C]
    [_ _ A B B C]
    [A A B C C _]
}
detect_shape! {
    gtr_tail_2_4
    [_ A B C C _]
    [_ _ A B B C]
    [A A B _ _ C]
}
detect_shape! {
    gtr_tail_2_5
    [_ _ _ _ C _]
    [_ A B C C _]
    [_ _ A B B C]
    [A A B _ _ C]
}
detect_shape! {
    gtr_tail_2_6
    [_ A B _ C _]
    [_ _ A B B C]
    [A A B _ C C]
}
detect_shape! {
    gtr_tail_2_7
    [_ A B C C _]
    [_ _ A B B C]
    [A A B _ C C]
}
detect_shape! {
    gtr_tail_3_1
    [_ A B _ C _]
    [_ _ A B B B]
    [A A B C C C]
}
detect_shape! {
    gtr_tail_3_2
    [_ A B _ _ C]
    [_ _ A B B B]
    [A A B C C C]
}
detect_shape! {
    gtr_tail_3_3
    [_ A B _ C C]
    [_ _ A B B B]
    [A A B C C C]
}
detect_shape! {
    gtr_tail_3_4
    [_ A B _ C C]
    [_ _ A B B B]
    [A A B C C _]
}
detect_shape! {
    gtr_tail_4_1
    [_ A B _ C _]
    [_ _ A B B C]
    [A A B C B C]
}
detect_shape! {
    gtr_tail_5_1
    [_ A B _ C _]
    [_ _ A B B C]
    [A A _ B C C]
}
detect_shape! {
    gtr_tail_5_2
    [_ A B C _ _]
    [_ _ A B B _]
    [A A C B C C]
}
detect_shape! {
    gtr_tail_6_1
    [_ A B _ C _]
    [_ _ A B B B]
    [A A _ C C C]
}
detect_shape! {
    gtr_tail_6_2
    [_ A B _ _ C]
    [_ _ A B B B]
    [A A _ C C C]
}
detect_shape! {
    gtr_tail_6_3
    [_ A B _ C C]
    [_ _ A B B B]
    [A A _ C C C]
}

// 多重側
detect_shape! {
    gtr_head_1
    [C C C _ _ _]
    [B A _ _ _ _]
    [B B A _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_head_2
    [D D D _ _ _]
    [C C C _ _ _]
    [B A _ _ _ _]
    [B B A _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_head_3
    [C D D _ _ _]
    [C C D _ _ _]
    [B A _ _ _ _]
    [B B A _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_head_4
    [C C D _ _ _]
    [C D D _ _ _]
    [B A _ _ _ _]
    [B B A _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_head_5
    [C D D _ _ _]
    [C C D _ _ _]
    [E E E _ _ _]
    [B A _ _ _ _]
    [B B A _ _ _]
    [A A _ _ _ _]
}
detect_shape! {
    gtr_head_6
    [C C D _ _ _]
    [C D D _ _ _]
    [E E E _ _ _]
    [B A _ _ _ _]
    [B B A _ _ _]
    [A A _ _ _ _]
}

#[cfg(test)]
mod tests {
    use puyoai::field::CoreField;

    use super::*;

    #[test]
    fn test_gtr_1() {
        assert_eq!(
            gtr_1(&CoreField::from_str(concat!(
                "      ", // 4
                "GRB   ", // 3
                "GGRB  ", // 2
                "RRBB  "  // 1
            ))),
            true
        );
        assert_eq!(
            gtr_1(&CoreField::from_str(concat!(
                "YYY   ", // 4
                "GRB   ", // 3
                "GGRB  ", // 2
                "RRBB  "  // 1
            ))),
            true
        );
        assert_eq!(
            gtr_1(&CoreField::from_str(concat!(
                "YYY   ", // 4
                "GRB   ", // 3
                "GGRB  ", // 2
                "RRBBB "  // 1: Bが余計にくっついてるので false
            ))),
            false
        );
        assert_eq!(
            gtr_1(&CoreField::from_str(concat!(
                "YYB   ", // 4: Bが余計にくっついてるので false
                "GRB   ", // 3
                "GGRB  ", // 2
                "RRBB  "  // 1
            ))),
            false
        );
        assert_eq!(
            gtr_1(&CoreField::from_str(concat!(
                "      ", // 4
                "GRB   ", // 3
                "GGRBB ", // 2
                "RRB   "  // 1
            ))),
            false
        );
    }
}
