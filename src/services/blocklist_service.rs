#![allow(dead_code)]

use crate::utils::http_client::{HttpClient, HttpClientError};
use futures::stream::{self, StreamExt};
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;
use tokio::time::{Duration, sleep};

#[derive(Deserialize, Debug)]
pub struct UrlLists {
    pub full: Vec<String>,
    pub msfw: Vec<String>,
    pub social: Vec<String>,
}

impl UrlLists {
    pub fn as_map(&self) -> HashMap<&str, &Vec<String>> {
        let mut map = HashMap::new();
        map.insert("full", &self.full);
        map.insert("msfw", &self.msfw);
        map.insert("social", &self.social);
        map
    }

    pub fn from_json(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json_data = fs::read_to_string(path)?;
        let lists: Self = serde_json::from_str(&json_data)?;
        Ok(lists)
    }
}

pub struct BlockListService {
    http_client: HttpClient,
    retries: usize,
}

impl BlockListService {
    pub fn new(client: HttpClient, retries: usize) -> Self {
        Self {
            http_client: client,
            retries,
        }
    }

    /// Faz download com retry simplificado
    pub async fn fetch_with_retry(&self, url: &str) -> Result<String, HttpClientError> {
        for attempt in 1..=self.retries {
            match self.http_client.get(url).await {
                Ok(resp) => match resp.text().await {
                    Ok(text) => return Ok(text),
                    Err(e) => eprintln!(
                        "Attempt {}/{} failed for {}: {}",
                        attempt, self.retries, url, e
                    ),
                },
                Err(e) => eprintln!(
                    "Attempt {}/{} failed for {}: {}",
                    attempt, self.retries, url, e
                ),
            }
            sleep(Duration::from_secs(2)).await;
        }

        Err(HttpClientError::IoError(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to fetch {} after {} retries", url, self.retries),
        )))
    }

    /// Sanitiza nome de arquivo de acordo com o sistema operacional
    fn sanitize_filename(url: &str) -> String {
        let mut name = url.replace("/", "_").replace("\\", "_");
        if cfg!(windows) {
            // substitui caracteres inválidos no Windows
            name = name.replace(
                |c: char| matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|'),
                "_",
            );
        }
        format!("full_{}", name)
    }

    /// Baixa URLs para arquivos temporários
    async fn download_to_temp(
        &self,
        urls: &[String],
        concurrency: usize,
        temp_dir: &Path,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        fs::create_dir_all(temp_dir)?;
        let mut temp_files = Vec::new();

        let stream = stream::iter(urls.iter().cloned())
            .map(|url| {
                let this = self;
                let temp_dir = temp_dir.to_path_buf();
                async move {
                    match this.fetch_with_retry(&url).await {
                        Ok(content) => {
                            let filename = BlockListService::sanitize_filename(&url);
                            let temp_path = temp_dir.join(format!("{}.tmp", filename));
                            let mut file = AsyncFile::create(&temp_path).await.ok()?;
                            file.write_all(content.as_bytes()).await.ok()?;
                            Some(temp_path)
                        }
                        Err(_) => None,
                    }
                }
            })
            .buffer_unordered(concurrency);

        futures::pin_mut!(stream);
        while let Some(opt_path) = stream.next().await {
            if let Some(path) = opt_path {
                temp_files.push(path);
            }
        }

        Ok(temp_files)
    }

    /// Consolida arquivos temporários, aplica regex, remove duplicatas e grava arquivo final
    fn consolidate_temp_files(
        &self,
        temp_files: Vec<PathBuf>,
        final_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let re = Regex::new(
            r"(?x)
            ^\s*
            (?:\|\|)?
            (?:0\.0\.0\.0\s+|127\.0\.0\.1\s+)?
            ([a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+)
            (?:\^)?
        ",
        )?;

        let mut domains = HashSet::new();
        for temp_file in temp_files {
            if let Ok(file) = File::open(&temp_file) {
                for line in BufReader::new(file).lines() {
                    if let Ok(l) = line {
                        if let Some(caps) = re.captures(&l) {
                            domains.insert(caps[1].to_string());
                        }
                    }
                }
            }
            let _ = fs::remove_file(&temp_file);
        }

        let mut sorted: Vec<_> = domains.into_iter().collect();
        sorted.sort();

        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut final_file = File::create(final_path)?;
        for domain in sorted {
            writeln!(final_file, "0.0.0.0 {}", domain)?;
        }

        Ok(())
    }

    /// Processa uma categoria
    pub async fn process_category(
        &self,
        category: &str,
        urls: &[String],
        concurrency: usize,
        output_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "🚀 Processando categoria: {} ({} URLs)",
            category,
            urls.len()
        );

        let temp_dir = output_dir.join("tmp");
        fs::create_dir_all(&temp_dir)?;

        let temp_files = self.download_to_temp(urls, concurrency, &temp_dir).await?;
        let final_path = output_dir.join(format!("{}.txt", category));
        self.consolidate_temp_files(temp_files, &final_path)?;

        Ok(())
    }

    /// Processa todas as categorias
    pub async fn fetch_all_categories(
        &self,
        lists: &UrlLists,
        concurrency: usize,
        output_dir: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = Path::new(output_dir);
        fs::create_dir_all(output_path)?;

        for (category, urls) in lists.as_map() {
            self.process_category(category, urls, concurrency, output_path)
                .await?;
        }

        Ok(())
    }
}
