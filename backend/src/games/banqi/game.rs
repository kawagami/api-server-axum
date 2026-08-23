//! 暗棋接上共用對戰框架：`impl GameEngine for BanqiGame`。
//!
//! 座位標籤用 first/second（顏色首翻才定，於事件中揭示）。

use serde_json::{json, Value};

use super::engine::{self, Action, BanqiState, Effect, Outcome, Piece, Side as BSide};
use crate::games::common::engine::{Applied, GameEngine, GameStatus, Side};

pub struct BanqiGame(BanqiState);

fn to_common(s: BSide) -> Side {
    match s {
        BSide::First => Side::First,
        BSide::Second => Side::Second,
    }
}

fn to_banqi(s: Side) -> BSide {
    match s {
        Side::First => BSide::First,
        Side::Second => BSide::Second,
    }
}

fn parse_xy(v: Option<&Value>) -> Option<(i8, i8)> {
    let arr = v?.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    Some((arr[0].as_i64()? as i8, arr[1].as_i64()? as i8))
}

fn piece_json(p: Piece) -> Value {
    json!({ "color": p.color.as_str(), "kind": p.kind.as_str() })
}

impl GameEngine for BanqiGame {
    const NAME: &'static str = "banqi";

    fn initial() -> Self {
        BanqiGame(engine::initial_state())
    }

    fn turn(&self) -> Side {
        to_common(self.0.turn)
    }

    fn side_label(side: Side) -> &'static str {
        match side {
            Side::First => "first",
            Side::Second => "second",
        }
    }

    fn try_move(&mut self, mover: Side, data: Option<&Value>) -> Result<Applied, String> {
        let action = match data.and_then(|d| d.get("action")).and_then(|v| v.as_str()) {
            Some("flip") => {
                let Some((col, row)) = parse_xy(data.and_then(|d| d.get("at"))) else {
                    return Err("bad_coord".into());
                };
                Action::Flip { col, row }
            }
            Some("move") => {
                let (Some(from), Some(to)) = (
                    parse_xy(data.and_then(|d| d.get("from"))),
                    parse_xy(data.and_then(|d| d.get("to"))),
                ) else {
                    return Err("bad_coord".into());
                };
                Action::Move { from, to }
            }
            _ => return Err("bad_action".into()),
        };

        let effect = engine::apply_action(&mut self.0, to_banqi(mover), action)
            .map_err(|e| e.to_string())?;

        let move_data = match effect {
            Effect::Flipped { col, row, piece } => json!({
                "action": "flip",
                "at": [col, row],
                "piece": piece_json(piece),
            }),
            Effect::Moved { from, to, captured } => json!({
                "action": "move",
                "from": [from.0, from.1],
                "to": [to.0, to.1],
                "captured": captured.map(piece_json),
            }),
        };

        Ok(Applied { move_data, extra: Vec::new() })
    }

    /// `{ flips: [[col,row]…], moves: { "col,row": [[col,row]…] } }`。
    ///
    /// 暗棋引擎沒有「枚舉合法步」的入口（只有 `apply_action` 這個驗證＋套用），
    /// 所以走法用**在複本上試走**取得 —— 規則仍然只有引擎那一份，不在這裡複刻位階/炮架。
    /// 盤面只有 4×8，一輪最多 16 子 × 32 格 = 512 次試走，成本可忽略。
    fn hints(&self) -> Option<Value> {
        let side = self.0.turn;
        // 尚未首翻定色時，只有翻子是合法的
        let color = engine::color_of(&self.0, side);

        let mut flips = Vec::new();
        let mut moves = serde_json::Map::new();
        for row in 0..4i8 {
            for col in 0..8i8 {
                match self.0.board[row as usize][col as usize] {
                    engine::Cell::Hidden(_) => flips.push(json!([col, row])),
                    engine::Cell::Up(p) if Some(p.color) == color => {
                        let mut targets = Vec::new();
                        for tr in 0..4i8 {
                            for tc in 0..8i8 {
                                if (tc, tr) == (col, row) {
                                    continue;
                                }
                                let mut probe = self.0.clone();
                                if engine::apply_action(
                                    &mut probe,
                                    side,
                                    Action::Move { from: (col, row), to: (tc, tr) },
                                )
                                .is_ok()
                                {
                                    targets.push(json!([tc, tr]));
                                }
                            }
                        }
                        if !targets.is_empty() {
                            moves.insert(format!("{col},{row}"), Value::Array(targets));
                        }
                    }
                    _ => {}
                }
            }
        }
        Some(json!({ "flips": flips, "moves": Value::Object(moves) }))
    }

    fn status(&self) -> GameStatus {
        match engine::status(&self.0) {
            Outcome::Continue => GameStatus::Ongoing,
            Outcome::Win { winner, reason } => GameStatus::Win {
                winner: to_common(winner),
                reason,
            },
            Outcome::Draw => GameStatus::Draw { reason: "draw_quiet" },
        }
    }
}
