use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub enum Perm {
    RoleRead,
    RoleAssign,
    RoleCreate,
    RoleUpdate,
    RoleDelete,
    MemberRead,
    WsRead,
    LogRead,
    AuditRead,
    BlogRead,
    BlogUpdate,
    BlogDelete,
    ImageRead,
    ImageCreate,
    ImageDelete,
    StockRead,
    StockUpdate,
    UserRead,
    UserCreate,
    UserDelete,
    SettingRead,
    SettingUpdate,
    TorrentRead,
    TorrentCreate,
    TorrentDelete,
    GameRead,
    InvoiceLotteryWrite,
    StatRead,
    GovTenderRead,
}

impl Perm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RoleRead    => "role:read",
            Self::RoleAssign  => "role:assign",
            Self::RoleCreate  => "role:create",
            Self::RoleUpdate  => "role:update",
            Self::RoleDelete  => "role:delete",
            Self::MemberRead  => "member:read",
            Self::WsRead      => "ws:read",
            Self::LogRead     => "log:read",
            Self::AuditRead   => "audit:read",
            Self::BlogRead    => "blog:read",
            Self::BlogUpdate  => "blog:update",
            Self::BlogDelete  => "blog:delete",
            Self::ImageRead   => "image:read",
            Self::ImageCreate => "image:create",
            Self::ImageDelete => "image:delete",
            Self::StockRead   => "stock:read",
            Self::StockUpdate => "stock:update",
            Self::UserRead       => "user:read",
            Self::UserCreate     => "user:create",
            Self::UserDelete     => "user:delete",
            Self::SettingRead    => "setting:read",
            Self::SettingUpdate  => "setting:update",
            Self::TorrentRead    => "torrent:read",
            Self::TorrentCreate  => "torrent:create",
            Self::TorrentDelete  => "torrent:delete",
            Self::GameRead       => "game:read",
            Self::InvoiceLotteryWrite => "invoice_lottery:write",
            Self::StatRead       => "stat:read",
            Self::GovTenderRead  => "gov_tender:read",
        }
    }
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Permission {
    pub id: i32,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct RoleWithPermissions {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Deserialize)]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct SetRolePermissions {
    pub permission_ids: Vec<i32>,
}

#[derive(Deserialize)]
pub struct SetUserRoles {
    pub role_ids: Vec<i32>,
}
