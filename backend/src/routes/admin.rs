use crate::{state::AppState, structs::features::Feature};
use axum::Router;

use super::{
    admin_blogs, admin_games, admin_gov_tenders, admin_invoice_lottery, admin_stats, admin_vocab,
    app_settings, audit_logs, auth, images, permissions, roles, stocks, torrents, users,
    with_feature,
};

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::new(state.clone()))
        .nest("/users", users::new(state.clone()))
        .nest("/roles", roles::new(state.clone()))
        .nest("/permissions", permissions::new(state.clone()))
        .nest("/audit_logs", audit_logs::new(state.clone()))
        .nest("/blogs", with_feature(state.clone(), Feature::Blog, admin_blogs::new(state.clone())))
        .nest("/images", with_feature(state.clone(), Feature::Blog, images::new(state.clone())))
        .nest("/stocks", with_feature(state.clone(), Feature::Stocks, stocks::new(state.clone())))
        .nest("/torrents", with_feature(state.clone(), Feature::Torrents, torrents::new(state.clone())))
        .nest("/games", with_feature(state.clone(), Feature::Games, admin_games::new(state.clone())))
        .nest("/gov_tenders", with_feature(state.clone(), Feature::GovTenders, admin_gov_tenders::new(state.clone())))
        .nest("/invoice_lottery_numbers", with_feature(state.clone(), Feature::Invoices, admin_invoice_lottery::new(state.clone())))
        .nest("/stats", admin_stats::new(state.clone()))
        .nest("/vocab", with_feature(state.clone(), Feature::Vocab, admin_vocab::new(state.clone())))
        .nest("/settings", app_settings::new(state))
}
