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
