mod app;
mod services;
mod utils;

use crate::app::BlockListApp;
use crate::utils::http_client::HttpClientBuilder;
use num_cpus;
use services::blocklist_service::{BlockListService, UrlLists};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::Mutex;

fn main() -> Result<(), Box<dyn Error>> {
    let max_threads = num_cpus::get();
    let rt = Builder::new_multi_thread()
        .worker_threads(max_threads)
        .enable_all()
        .build()?;

    rt.block_on(async {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "*/*".to_string());

        let client = HttpClientBuilder::new()
            .user_agent("User-Agent Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0")
            .headers(headers)
            .timeout(Duration::from_secs(120))
            .build()?;

        let service = Arc::new(Mutex::new(BlockListService::new(client, 5)));
        let lists = Arc::new(UrlLists::from_json("./data/lists.json")?);

        let app = Arc::new(BlockListApp::new(service, lists, "./output".into()));

        app.clone().start_updater(60 * 60 * 24);

        app.clone().start_api(3000).await?;

        Ok::<(), Box<dyn Error>>(())
    })?;

    Ok(())
}
