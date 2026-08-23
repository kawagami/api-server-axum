//! 圍棋接上共用對戰框架：`impl GameEngine for GoGame`。

use serde_json::{json, Value};

use super::engine::{self, GoState, Outcome, Stone};
use crate::games::common::engine::{Applied, GameEngine, GameStatus, Side};

pub struct GoGame(GoState);

fn to_common(s: Stone) -> Side {
    match s {
        Stone::Black => Side::First,
        Stone::White => Side::Second,
    }
}

fn parse_at(v: Option<&Value>) -> Option<(i8, i8)> {
    let arr = v?.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    Some((arr[0].as_i64()? as i8, arr[1].as_i64()? as i8))
}

impl GameEngine for GoGame {
    const NAME: &'static str = "go";

    fn initial() -> Self {
        GoGame(engine::initial_state())
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
        // 虛手
        if data.and_then(|d| d.get("pass")).and_then(|p| p.as_bool()) == Some(true) {
            engine::pass(&mut self.0);
            return Ok(Applied {
                move_data: json!({ "pass": true, "by": Self::side_label(mover) }),
                extra: Vec::new(),
            });
        }
        // 落子
        let Some((c, r)) = parse_at(data.and_then(|d| d.get("at"))) else {
            return Err("bad_coord".into());
        };
        engine::is_legal(&self.0, c, r).map_err(|e| e.as_str().to_string())?;
        let captured = engine::apply(&mut self.0, c, r);
        let captured_json: Vec<Value> = captured.iter().map(|&(cc, rr)| json!([cc, rr])).collect();
        Ok(Applied {
            move_data: json!({
                "at": [c, r],
                "by": Self::side_label(mover),
                "captured": captured_json,
            }),
            extra: Vec::new(),
        })
    }

    /// `{ forbidden: [[col,row]…] }` —— **空點裡不能下的那些**（自殺 / 劫）。
    /// 不送「可下點」是因為那幾乎是整盤空點（19×19 一張 3KB 的清單，每手兩份），
    /// 而「空不空」前端自己看得出來，唯一需要 server 判的就是這兩條規則。
    fn hints(&self) -> Option<Value> {
        let mut forbidden = Vec::new();
        for r in 0..engine::SIZE {
            for c in 0..engine::SIZE {
                if self.0.board[r as usize][c as usize].is_some() {
                    continue;
                }
                if engine::is_legal(&self.0, c, r).is_err() {
                    forbidden.push(json!([c, r]));
                }
            }
        }
        Some(json!({ "forbidden": forbidden }))
    }

    fn status(&self) -> GameStatus {
        match engine::status(&self.0) {
            Outcome::Continue => GameStatus::Ongoing,
            Outcome::Win(stone) => GameStatus::Win {
                winner: to_common(stone),
                reason: "score",
            },
        }
    }
}
