use crate::state::AppState;
use axum::Router;

use super::{
    admin_blogs, admin_games, admin_gov_tenders, admin_invoice_lottery, admin_stats, admin_vocab,
    app_settings, audit_logs, auth, images, permissions, roles, stocks, torrents, users,
};

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::new(state.clone()))
        .nest("/users", users::new(state.clone()))
        .nest("/roles", roles::new(state.clone()))
        .nest("/permissions", permissions::new(state.clone()))
        .nest("/audit_logs", audit_logs::new(state.clone()))
        .nest("/blogs", admin_blogs::new(state.clone()))
        .nest("/images", images::new(state.clone()))
        .nest("/stocks", stocks::new(state.clone()))
        .nest("/torrents", torrents::new(state.clone()))
        .nest("/games", admin_games::new(state.clone()))
        .nest("/gov_tenders", admin_gov_tenders::new(state.clone()))
        .nest("/invoice_lottery_numbers", admin_invoice_lottery::new(state.clone()))
        .nest("/stats", admin_stats::new(state.clone()))
        .nest("/vocab", admin_vocab::new(state.clone()))
        .nest("/settings", app_settings::new(state))
}
