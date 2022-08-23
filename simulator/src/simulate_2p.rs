/*
   TODO
   - 同フレームのやつが2個（1Pと2P）並んでたら、後にある方を処理する前に盤面の更新を行わないこと（同時なので）
   - 同時に死んだら引き分けにするべき
   - 全消しの考慮
*/

use std::{
    collections::BinaryHeap,
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
};

use chrono::{DateTime, Utc};
use cpu::bot::{PlayerState, AI};
use logger::Logger;
use puyoai::{
    decision::Decision, es_field::EsCoreField, es_frame, field::CoreField, field_bit::FieldBit,
    rensa_tracker::RensaNonTracker,
};
use serde::{Deserialize, Serialize};

use super::{
    convert::{convert_core_field, convert_kumipuyo_seq},
    haipuyo_detector::*,
};

// TODO: マージンの実装
const OJAMA_PUYO_RATE: usize = 70;

pub fn simulate_2p(
    logger: &mut Box<dyn Logger>,
    ai_1p: &Box<dyn AI>,
    ai_2p: &Box<dyn AI>,
    win_goal: usize, // 何本先取か
    visible_tumos: usize,
    haipuyo_margin: Option<usize>, // Noneならランダムに、Someならその番号から順番に使う
) -> Result<SimulateResult2P, std::io::Error> {
    // お互いの勝利数
    let mut win_count_1p: usize = 0;
    let mut win_count_2p: usize = 0;

    // 各試合の詳細
    let mut json_matches: Vec<JsonMatch> = vec![];

    while win_count_1p < win_goal && win_count_2p < win_goal {
        // 配ぷよを決める
        let seq = match haipuyo_margin {
            None => HaipuyoDetector::random_haipuyo(),
            Some(margin) => HaipuyoDetector::retrieve_haipuyo(
                (margin + win_count_1p + win_count_2p) % TUMO_PATTERN,
            ),
        };

        // 各プレイヤーの状態
        let mut player_state_1p = PlayerState::initial_state(vec![], Some(seq.clone()));
        let mut player_state_2p = PlayerState::initial_state(vec![], Some(seq.clone()));
        player_state_1p.set_seq(visible_tumos);
        player_state_2p.set_seq(visible_tumos);

        // この試合で起きたイベント
        let mut json_events: Vec<JsonEvent> = vec![];
        // どっちが勝ったか
        let winner_player: Option<Player>;

        // 初期盤面をpush
        json_events.push(JsonEvent {
            frame: 0,
            json_state_1p: player_state_1p.clone().into(),
            json_state_2p: player_state_2p.clone().into(),
        });

        // 処理すべき各イベント
        let mut events = BinaryHeap::new();
        events.push(Event::new(0, Player::One, None, false));
        events.push(Event::new(0, Player::Two, None, false));

        // どちらかが死ぬまで続ける
        'battle: loop {
            let event = match events.pop() {
                Some(event) => event,
                None => unreachable!(),
            };

            let (player_state_myself, player_state_opponent) = if event.player == Player::One {
                (&mut player_state_1p, &mut player_state_2p)
            } else {
                (&mut player_state_2p, &mut player_state_1p)
            };
            let ai = if event.player == Player::One {
                ai_1p
            } else {
                ai_2p
            };

            // `json_events` を更新
            // TODO: 関数化できるならそうしたい
            macro_rules! push_json_event {
                ($frame:expr, $player:expr) => {
                    json_events.push(JsonEvent {
                        frame: $frame,
                        json_state_1p: match $player {
                            Player::One => player_state_myself.clone(),
                            Player::Two => player_state_opponent.clone(),
                        }
                        .into(),
                        json_state_2p: match $player {
                            Player::One => player_state_opponent.clone(),
                            Player::Two => player_state_myself.clone(),
                        }
                        .into(),
                    });
                };
            }

            // 置く場所がすでに決まっている or 連鎖中
            if let Some(decision) = event.decision {
                // 連鎖中でないなら、ぷよを置いて `PlayerState` を更新する
                if player_state_myself.current_chain == 0 {
                    player_state_myself.drop_kumipuyo(&decision);
                    player_state_myself.tumo_index += 1;
                    player_state_myself.set_seq(visible_tumos);
                    player_state_myself.frame = event.frame;
                }

                // ぷよを置いた後の盤面を push
                push_json_event!(event.frame, event.player);

                // 連鎖が発生したら、盤面・フレーム・おじゃまを更新
                let (chain_score, chain_frame) = vanish_single_chain(
                    &mut player_state_myself.field,
                    player_state_myself.current_chain + 1,
                );
                if chain_score != 0 {
                    // おじゃまを 1 連鎖分処理
                    player_state_myself.carry_over += chain_score;
                    let mut ojama = player_state_myself.carry_over / OJAMA_PUYO_RATE;
                    player_state_myself.carry_over %= OJAMA_PUYO_RATE;

                    // 相殺に当てる
                    if ojama > 0 && player_state_myself.fixed_ojama > 0 {
                        let pay = ojama.min(player_state_myself.fixed_ojama);
                        ojama -= pay;
                        player_state_myself.fixed_ojama -= pay;
                    }
                    if ojama > 0 && player_state_myself.pending_ojama > 0 {
                        let pay = ojama.min(player_state_myself.pending_ojama);
                        ojama -= pay;
                        player_state_myself.pending_ojama -= pay;
                    }

                    // 余った分は相手に送る
                    if ojama > 0 {
                        player_state_opponent.pending_ojama += ojama;
                    }

                    // 自身の状態を更新
                    player_state_myself.score += chain_score;
                    player_state_myself.frame += chain_frame;
                    player_state_myself.current_chain += 1;

                    events.push(Event::new(
                        player_state_myself.frame,
                        event.player,
                        Some(decision),
                        false,
                    ));

                    continue;
                }
            }

            // 連鎖が発生していないので 0 に戻す
            player_state_myself.current_chain = 0;

            // おじゃまを降らせる
            if !event.force_think && player_state_myself.fixed_ojama > 0 {
                // 30個以上だったら30個、そうでないならその分降らせる
                let ojama = if player_state_myself.fixed_ojama >= 30 {
                    30
                } else {
                    player_state_myself.fixed_ojama
                };
                player_state_myself.fixed_ojama -= ojama;

                let ojama_drop_frame = player_state_myself.field.es_drop_ojama(
                    ojama,
                    Some(
                        ((win_count_1p
                            + win_count_2p
                            + player_state_myself.score
                            + player_state_opponent.score
                            + ojama)
                            & 0b11111111) as u8,
                    ),
                );

                // フレームを更新
                // TODO: おじゃまの降る位置がかなり早い段階（降り始めたタイミング）で凝視できるようになってしまっている
                player_state_myself.frame += ojama_drop_frame;

                events.push(Event::new(
                    player_state_myself.frame,
                    event.player,
                    None,
                    true, // おじゃまが降ったので、次は必ず操作を行う
                ));

                // おじゃまが降った後の盤面を push
                push_json_event!(event.frame, event.player);

                continue;
            }

            // ぷよを置いて / おじゃまが降って、自陣が死んでたら終了
            // TODO: 同時に死ぬ場合がありうる
            if player_state_myself.field.is_dead() {
                winner_player = Some(event.player.opponent());
                break 'battle;
            }

            // 思考する前に相手の予告ぷよを確定させる
            if player_state_opponent.pending_ojama > 0 {
                player_state_opponent.fixed_ojama += player_state_opponent.pending_ojama;
                player_state_opponent.pending_ojama = 0;
            }

            // AIで思考する
            let ai_decision = ai.think(
                player_state_myself.clone(),
                Some(player_state_opponent.clone()),
            );
            let decision = ai_decision.decisions[0].clone();
            events.push(Event::new(
                // そこに置くのに必要なフレーム数を加算
                event.frame + player_state_myself.field.es_frames_to_drop_next(&decision),
                event.player,
                Some(decision),
                false,
            ));
        }

        // 結果を更新
        match winner_player {
            Some(player) => match player {
                Player::One => {
                    logger.print("1P won! ".into())?;
                    win_count_1p += 1;
                }
                Player::Two => {
                    logger.print("2P won! ".into())?;
                    win_count_2p += 1;
                }
            },
            None => unreachable!(),
        }
        logger.print(format!(
            "{:3} vs {:3} ({:6} - {:6})\n",
            win_count_1p, win_count_2p, player_state_1p.score, player_state_2p.score
        ))?;

        // この試合の結果をpush
        json_matches.push(JsonMatch {
            won_1p: winner_player.unwrap() == Player::One,
            tumos: convert_kumipuyo_seq(&seq),
            json_events,
        })
    }

    logger.print(format!(
        "Result: {:3} vs {:3}\n",
        win_count_1p, win_count_2p
    ))?;

    Ok(SimulateResult2P::new(
        win_count_1p,
        win_count_2p,
        visible_tumos,
        json_matches,
    ))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonState {
    pub tumo_index: usize,
    pub field: String, // pfen-like
    pub score: usize,
    pub ojama_fixed: usize,   // 確定おじゃまぷよ
    pub ojama_ongoing: usize, // 予告おじゃまぷよ
    pub current_chain: usize, // 現在の連鎖数
}

impl From<PlayerState> for JsonState {
    fn from(player_state: PlayerState) -> Self {
        Self {
            tumo_index: player_state.tumo_index,
            field: convert_core_field(&player_state.field),
            score: player_state.score,
            ojama_fixed: player_state.fixed_ojama,
            ojama_ongoing: player_state.pending_ojama,
            current_chain: player_state.current_chain,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonEvent {
    pub frame: usize,
    pub json_state_1p: JsonState,
    pub json_state_2p: JsonState,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonMatch {
    pub won_1p: bool,
    pub tumos: Vec<String>,
    pub json_events: Vec<JsonEvent>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulateResult2P {
    pub date: DateTime<Utc>,
    pub win_count_1p: usize,
    pub win_count_2p: usize,
    pub visible_tumos: usize,
    pub json_matches: Vec<JsonMatch>,
}

impl SimulateResult2P {
    fn new(
        win_count_1p: usize,
        win_count_2p: usize,
        visible_tumos: usize,
        json_matches: Vec<JsonMatch>,
    ) -> Self {
        SimulateResult2P {
            date: Utc::now(),
            win_count_1p,
            win_count_2p,
            visible_tumos,
            json_matches,
        }
    }

    pub fn export_json(
        &self,
        pr_number: usize,
        ai_name_1p: &str,
        ai_name_2p: &str,
    ) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_string(&self).unwrap();

        let file_dir = format!(
            "kifus/simulator_2p/{}_{}_vs_{}",
            pr_number, &ai_name_1p, &ai_name_2p
        );
        create_dir_all(&file_dir)?;

        let time_text = Utc::now().format("%Y%m%d_%H%M%S_%f");
        let file_path = format!("{}/{}.json", &file_dir, &time_text);
        let mut buf_writer = BufWriter::new(File::create(&file_path)?);

        write!(buf_writer, "{}", &serialized)?;
        buf_writer.flush()?;

        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Player {
    One,
    Two,
}

impl Player {
    fn opponent(&self) -> Self {
        match *self {
            Player::One => Player::Two,
            Player::Two => Player::One,
        }
    }
}

struct Event {
    /// 現在のフレーム数
    frame: usize,
    /// プレイヤー
    player: Player,
    /// どこに置くか
    decision: Option<Decision>,
    /// おじゃまを無視してthinkするか
    force_think: bool,
}

impl Event {
    fn new(frame: usize, player: Player, decision: Option<Decision>, force_think: bool) -> Self {
        Self {
            frame,
            player,
            decision,
            force_think,
        }
    }
}

// `BinaryHeap` に突っ込むため
impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.frame.cmp(&self.frame)
    }
}
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.frame == other.frame
    }
}
impl Eq for Event {}

/// 1 連鎖分進めて (点数, フレーム数) を返す
fn vanish_single_chain(cf: &mut CoreField, current_chain: usize) -> (usize, usize) {
    let escaped = cf.field_mut().escape_invisible();
    let mut erased = unsafe { FieldBit::uninitialized() };
    let chain_score = cf.field().vanish(
        current_chain,
        &mut erased,
        &mut puyoai::rensa_tracker::RensaNonTracker::new(),
    );

    // 連鎖が発生しなかった
    if chain_score == 0 {
        cf.field_mut().recover_invisible(&escaped);
        return (0, 0);
    }

    let max_drops = unsafe {
        cf.field_mut()
            .drop_after_vanish(erased, &mut RensaNonTracker::new())
    };
    cf.update_height();
    let chain_frame = es_frame::FRAMES_CHAIN[max_drops];

    cf.field_mut().recover_invisible(&escaped);
    (chain_score, chain_frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vanish_single_chain() {
        let mut cf = CoreField::from_str(concat!(
            "G.....", // 10
            "Y.....", // 9
            "GR...B", // 8
            "RR...B", // 7
            "GRR..Y", // 6
            "GGRY.Y", // 5
            "YYYBGG", // 4
            "GRBGYG", // 3
            "GGRBBY", // 2
            "RRBYYG"  // 1
        ));

        let (score, frame) = vanish_single_chain(&mut cf, 1);
        assert_eq!(score, 180);
        assert_eq!(frame, 80);

        let cf_expected = CoreField::from_str(concat!(
            "G.....", // 9
            "Y....B", // 8
            "G....B", // 7
            "G....Y", // 6
            "GG.Y.Y", // 5
            "YYYBGG", // 4
            "GRBGYG", // 3
            "GGRBBY", // 2
            "RRBYYG"  // 1
        ));
        assert_eq!(cf, cf_expected);
    }
}
