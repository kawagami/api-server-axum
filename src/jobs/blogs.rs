use std::collections::HashSet;

use crate::{
    repositories::{
        blogs::get_blogs_with_pagination,
        firebase::{delete, images},
    },
    state::AppStateV2,
    structs::{firebase::DeleteImageRequest, jobs::AppJob},
};
use async_trait::async_trait;
use regex::Regex;

#[derive(Clone)]
pub struct ActiveImageJob;

#[async_trait]
impl AppJob for ActiveImageJob {
    fn cron_expression(&self) -> &str {
        "0 0 * * * *"
    }

    // 每小時清除 blogs 中沒有在使用的圖片
    async fn run(&self, state: AppStateV2) {
        // 取得所有 blogs
        let blogs = match get_blogs_with_pagination(&state, 1000, 0).await {
            Ok(blogs) => blogs,
            Err(err) => {
                tracing::error!("{}", err);
                vec![]
            }
        };

        // 定義 regex 抓取圖片路徑
        let image_regex = Regex::new(r"!\[[^\]]*\]\(([^)]+)\)").unwrap();

        // 提取所有圖片路徑並合併成 HashSet<String>
        let image_paths: HashSet<String> = blogs
            .into_iter()
            .flat_map(|blog| {
                let markdown = blog.markdown; // 移動 blog.markdown 的所有權
                image_regex
                    .captures_iter(&markdown)
                    .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                    .collect::<HashSet<String>>()
            })
            .collect();

        // 取所有 fastapi-upload 的圖片
        let all_images = match images(&state).await {
            Ok(images) => images,
            Err(err) => {
                tracing::error!("{}", err);
                vec![]
            }
        };

        // 對所有圖片檢查是否在 blogs 中有使用
        for image in &all_images {
            if !image_paths.contains(&image.url) {
                // 整理為 delete API 的格式
                let delete_data = DeleteImageRequest {
                    file_name: image.name.to_owned(),
                };
                // 執行動作
                let _ = delete(&state, delete_data).await;
            }
        }
    }
}
