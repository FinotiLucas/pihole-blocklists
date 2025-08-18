use crate::services::blocklist_service::{BlockListService, UrlLists};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use std::time::Duration;
use std::{fs, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

pub struct BlockListApp {
    service: Arc<Mutex<BlockListService>>,
    lists: Arc<UrlLists>,
    output_dir: String,
}

impl BlockListApp {
    pub fn new(
        service: Arc<Mutex<BlockListService>>,
        lists: Arc<UrlLists>,
        output_dir: String,
    ) -> Self {
        Self {
            service,
            lists,
            output_dir,
        }
    }

    pub fn start_updater(self: Arc<Self>, interval_secs: u64) {
        let service = self.service.clone();
        let lists = self.lists.clone();
        let output_dir = self.output_dir.clone();

        tokio::spawn(async move {
            loop {
                {
                    let service = service.lock().await;
                    println!(" Atualizando listas...");
                    if let Err(e) = service.fetch_all_categories(&lists, 10, &output_dir).await {
                        eprintln!("Erro ao atualizar listas: {}", e);
                    } else {
                        println!(" Listas atualizadas");
                    }
                }
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
        });
    }

    pub async fn start_api(self: Arc<Self>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await?;

        let app = Router::new()
            .route("/files/:category", get(Self::get_file))
            .with_state(self.clone());

        println!("Servidor rodando em {}", addr);

        axum::serve(listener, app).await?;

        Ok(())
    }

    async fn get_file(
        State(app): State<Arc<Self>>,
        Path(category): Path<String>,
    ) -> impl IntoResponse {
        let path = format!("{}/{}.txt", app.output_dir, category);

        if let Ok(content) = fs::read_to_string(&path) {
            (StatusCode::OK, content).into_response()
        } else {
            (StatusCode::NOT_FOUND, "Arquivo não encontrado").into_response()
        }
    }
}
