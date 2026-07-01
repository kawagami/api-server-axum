//! 西洋棋接上共用對戰框架：`impl GameEngine for WesternChessGame`。

use serde_json::{json, Value};

use super::engine;
use super::types::{Castle, Color, Move, PieceKind, State};
use crate::games::common::engine::{Applied, GameEngine, GameStatus, Side};

pub struct WesternChessGame(State);

fn to_common(c: Color) -> Side {
    match c {
        Color::White => Side::First,
        Color::Black => Side::Second,
    }
}

fn parse_coord(v: Option<&Value>) -> Option<(i8, i8)> {
    let arr = v?.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    let c = arr[0].as_i64()?;
    let r = arr[1].as_i64()?;
    ((0..8).contains(&c) && (0..8).contains(&r)).then_some((c as i8, r as i8))
}

fn parse_promo(v: Option<&Value>) -> Result<Option<PieceKind>, ()> {
    match v.and_then(|d| d.get("promo")).and_then(|p| p.as_str()) {
        None => Ok(None),
        Some("q") => Ok(Some(PieceKind::Queen)),
        Some("r") => Ok(Some(PieceKind::Rook)),
        Some("b") => Ok(Some(PieceKind::Bishop)),
        Some("n") => Ok(Some(PieceKind::Knight)),
        Some(_) => Err(()),
    }
}

fn promo_str(k: PieceKind) -> &'static str {
    match k {
        PieceKind::Queen => "q",
        PieceKind::Rook => "r",
        PieceKind::Bishop => "b",
        PieceKind::Knight => "n",
        _ => "q",
    }
}

impl GameEngine for WesternChessGame {
    const NAME: &'static str = "western_chess";

    fn initial() -> Self {
        WesternChessGame(engine::initial_state())
    }

    fn turn(&self) -> Side {
        to_common(engine::turn(&self.0))
    }

    fn side_label(side: Side) -> &'static str {
        match side {
            Side::First => "white",
            Side::Second => "black",
        }
    }

    fn try_move(&mut self, _mover: Side, data: Option<&Value>) -> Result<Applied, String> {
        let (Some(from), Some(to)) = (
            parse_coord(data.and_then(|d| d.get("from"))),
            parse_coord(data.and_then(|d| d.get("to"))),
        ) else {
            return Err("bad_coord".into());
        };
        let promo = parse_promo(data).map_err(|_| "bad_promo".to_string())?;

        // 來源格檢查（輪次已由 harness 確認）
        match self.0.board[from.1 as usize][from.0 as usize] {
            None => return Err("no_piece".into()),
            Some(p) if p.color != self.0.turn => return Err("wrong_piece".into()),
            _ => {}
        }

        // 在合法步中比對 from/to（＋升變）
        let candidates: Vec<Move> = engine::legal_moves(&self.0)
            .into_iter()
            .filter(|mv| mv.from == from && mv.to == to)
            .collect();
        if candidates.is_empty() {
            return Err("illegal".into());
        }
        let chosen = if candidates[0].promo.is_some() {
            // 升變：promo 缺省為后
            let want = promo.unwrap_or(PieceKind::Queen);
            candidates
                .into_iter()
                .find(|mv| mv.promo == Some(want))
                .ok_or("bad_promo".to_string())?
        } else {
            candidates[0]
        };

        let info = engine::apply(&mut self.0, chosen);

        let mut md = serde_json::Map::new();
        md.insert("from".into(), json!([from.0, from.1]));
        md.insert("to".into(), json!([to.0, to.1]));
        if let Some(k) = info.promo {
            md.insert("promo".into(), json!(promo_str(k)));
        }
        if let Some(castle) = info.castle {
            md.insert("castle".into(), json!(match castle {
                Castle::King => "king",
                Castle::Queen => "queen",
            }));
        }
        if let Some((c, r)) = info.ep_capture {
            md.insert("ep_capture".into(), json!([c, r]));
        }

        let mut extra = Vec::new();
        if matches!(engine::status(&self.0), engine::Outcome::Continue)
            && engine::is_in_check(&self.0, self.0.turn)
        {
            extra.push(("check", json!({ "side": Self::side_label(to_common(self.0.turn)) })));
        }

        Ok(Applied { move_data: Value::Object(md), extra })
    }

    fn status(&self) -> GameStatus {
        match engine::status(&self.0) {
            engine::Outcome::Continue => GameStatus::Ongoing,
            engine::Outcome::Checkmate { winner } => GameStatus::Win {
                winner: to_common(winner),
                reason: "checkmate",
            },
            engine::Outcome::Stalemate => GameStatus::Draw { reason: "stalemate" },
            engine::Outcome::Draw => GameStatus::Draw { reason: "draw" },
        }
    }
}
