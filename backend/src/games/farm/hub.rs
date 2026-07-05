//! 農場經營房型參數。大廳 / 房間狀態走 `common::room` 泛型框架。2–4 人，重啟即丟失。

use super::engine::GameState;
use crate::games::common::room::{RoomHub, RoomKind};

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 4;

/// marker type：impl `RoomKind` 提供農場的靜態參數。
pub struct FarmRoom;

impl RoomKind for FarmRoom {
    type Playing = GameState;
    type Options = ();

    const NAME: &'static str = super::NAME;
    const MIN_PLAYERS: usize = MIN_PLAYERS;
    const MAX_PLAYERS: usize = MAX_PLAYERS;
    // 等待中有人離開會重編號，room_update 逐人帶當前 seat
    const SEAT_IN_ROOM_UPDATE: bool = true;

    fn default_room_name(id: u64) -> String {
        format!("農場 #{id}")
    }
}

pub type FarmHub = RoomHub<FarmRoom>;
pub type Room = crate::games::common::room::Room<FarmRoom>;
pub use crate::games::common::room::RoomState;
