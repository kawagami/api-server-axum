//! 五子棋接上共用對戰框架：`impl GameEngine for GomokuGame`。

use serde_json::{json, Value};

use super::engine::{self, GomokuState, Outcome, Stone};
use crate::games::common::engine::{Applied, GameEngine, GameStatus, Side};

pub struct GomokuGame(GomokuState);

fn to_common(s: Stone) -> Side {
    match s {
        Stone::Black => Side::First,
        Stone::White => Side::Second,
    }
}

/// 解析 `[col, row]`。
fn parse_at(v: Option<&Value>) -> Option<(i8, i8)> {
    let arr = v?.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    Some((arr[0].as_i64()? as i8, arr[1].as_i64()? as i8))
}

impl GameEngine for GomokuGame {
    const NAME: &'static str = "gomoku";

    fn initial() -> Self {
        GomokuGame(engine::initial_state())
    }

    fn turn(&self) -> Side {
        to_common(self.0.turn)
    }

    fn side_label(side: Side) -> &'static str {
        match side {
            Side::First => "black",
            Side::Second => "white",
        }
    }

    fn try_move(&mut self, mover: Side, data: Option<&Value>) -> Result<Applied, String> {
        let Some((col, row)) = parse_at(data.and_then(|d| d.get("at"))) else {
            return Err("bad_coord".into());
        };
        engine::is_legal(&self.0, col, row).map_err(|e| e.to_string())?;
        engine::apply(&mut self.0, col, row);
        Ok(Applied {
            move_data: json!({ "at": [col, row], "by": Self::side_label(mover) }),
            extra: Vec::new(),
        })
    }

    fn status(&self) -> GameStatus {
        match engine::status(&self.0) {
            Outcome::Continue => GameStatus::Ongoing,
            Outcome::Win(stone) => GameStatus::Win {
                winner: to_common(stone),
                reason: "five_in_row",
            },
            Outcome::Draw => GameStatus::Draw { reason: "draw_full" },
        }
    }
}
