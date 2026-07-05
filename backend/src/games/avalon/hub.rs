//! 阿瓦隆房型參數。大廳 / 房間狀態走 `common::room` 泛型框架。N 人房，重啟即丟失。

use serde_json::{json, Map, Value};

use super::engine::AvalonState;
use super::roles::Options;
use crate::games::common::room::{RoomHub, RoomHubInner, RoomKind};

pub const MIN_PLAYERS: usize = 5;
pub const MAX_PLAYERS: usize = 10;

/// marker type：impl `RoomKind` 提供阿瓦隆的靜態參數。
pub struct AvalonRoom;

impl RoomKind for AvalonRoom {
    type Playing = AvalonState;
    type Options = Options;

    const NAME: &'static str = super::NAME;
    const MIN_PLAYERS: usize = MIN_PLAYERS;
    const MAX_PLAYERS: usize = MAX_PLAYERS;

    fn default_room_name(id: u64) -> String {
        format!("房 #{id}")
    }

    fn parse_options(data: Option<&Value>) -> Options {
        Options {
            mordred: data.and_then(|d| d.pointer("/options/mordred")).and_then(|v| v.as_bool()).unwrap_or(false),
            oberon: data.and_then(|d| d.pointer("/options/oberon")).and_then(|v| v.as_bool()).unwrap_or(false),
        }
    }

    fn extend_room_snapshot(options: &Options, obj: &mut Map<String, Value>) {
        obj.insert("options".into(), json!({ "mordred": options.mordred, "oberon": options.oberon }));
    }
}

pub type AvalonHub = RoomHub<AvalonRoom>;
pub type AvalonHubInner = RoomHubInner<AvalonRoom>;
pub type Room = crate::games::common::room::Room<AvalonRoom>;
pub use crate::games::common::room::RoomState;
