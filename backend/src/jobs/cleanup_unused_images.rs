use crate::{services::images as images_service, state::AppState};

pub async fn run(state: AppState) {
    images_service::cleanup_unused_images(state.get_pool(), state.get_storage()).await;
}
