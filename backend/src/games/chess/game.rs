//! 象棋接上共用對戰框架：`impl GameEngine for ChessGame`。純引擎在 `engine`/`types`。

use serde_json::{json, Map, Value};

use super::engine;
use super::types::{GameState, Move, Side as ChSide, Square, Status};
use crate::games::common::engine::{Applied, GameEngine, GameStatus, Side};

pub struct ChessGame(GameState);

fn to_common(s: ChSide) -> Side {
    match s {
        ChSide::Red => Side::First,
        ChSide::Black => Side::Second,
    }
}

/// 解析 `[col, row]`，越界或非整數回 None。
fn parse_square(v: Option<&Value>) -> Option<Square> {
    let arr = v?.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    let col = arr[0].as_i64()?;
    let row = arr[1].as_i64()?;
    let sq = Square::new(col as i8, row as i8);
    sq.in_bounds().then_some(sq)
}

fn sq_json(sq: Square) -> Value {
    json!([sq.col, sq.row])
}

impl GameEngine for ChessGame {
    const NAME: &'static str = "chess";

    fn initial() -> Self {
        ChessGame(engine::initial_state())
    }

    fn turn(&self) -> Side {
        to_common(self.0.turn)
    }

    fn side_label(side: Side) -> &'static str {
        match side {
            Side::First => "red",
            Side::Second => "black",
        }
    }

    fn try_move(&mut self, _mover: Side, data: Option<&Value>) -> Result<Applied, String> {
        let (Some(from), Some(to)) = (
            parse_square(data.and_then(|d| d.get("from"))),
            parse_square(data.and_then(|d| d.get("to"))),
        ) else {
            return Err("bad_coord".into());
        };
        let mv = Move { from, to };
        engine::is_legal(&self.0, mv).map_err(|r| r.code().to_string())?;
        engine::apply(&mut self.0, mv);

        let mut extra = Vec::new();
        if matches!(engine::game_status(&self.0), Status::Ongoing)
            && engine::is_in_check(&self.0, self.0.turn)
        {
            extra.push((
                "check",
                json!({ "side": Self::side_label(to_common(self.0.turn)) }),
            ));
        }

        Ok(Applied {
            move_data: json!({ "from": sq_json(from), "to": sq_json(to) }),
            extra,
        })
    }

    /// `{ moves: { "col,row": [[col,row]…] } }` —— 只列輪到的一方、且真的有走法的子。
    /// 走法枚舉直接呼叫引擎的 `legal_moves`（含送將過濾），所以提示與判定同一份規則。
    fn hints(&self) -> Option<Value> {
        let mut moves = Map::new();
        for row in 0..10i8 {
            for col in 0..9i8 {
                match self.0.board[row as usize][col as usize] {
                    Some(p) if p.side == self.0.turn => {
                        let targets: Vec<Value> =
                            engine::legal_moves(&self.0, Square::new(col, row))
                                .into_iter()
                                .map(sq_json)
                                .collect();
                        if !targets.is_empty() {
                            moves.insert(format!("{col},{row}"), Value::Array(targets));
                        }
                    }
                    _ => {}
                }
            }
        }
        Some(json!({ "moves": Value::Object(moves) }))
    }

    fn status(&self) -> GameStatus {
        match engine::game_status(&self.0) {
            Status::Ongoing => GameStatus::Ongoing,
            Status::Checkmate { winner } => GameStatus::Win {
                winner: to_common(winner),
                reason: "checkmate",
            },
            Status::Stalemate { loser } => GameStatus::Win {
                winner: to_common(loser.opponent()),
                reason: "stalemate",
            },
            Status::Draw => GameStatus::Draw { reason: "draw_60" },
        }
    }
}
