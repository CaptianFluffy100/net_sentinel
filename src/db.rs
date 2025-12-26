use crate::models::{Isp, Website, GameServer};
use crate::out;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Database {
    pub isps: Vec<Isp>,
    pub websites: Vec<Website>,
    pub game_servers: Vec<GameServer>,
    #[serde(skip)]
    next_id: i64,
}

impl Database {
    pub fn get_next_id(&mut self) -> i64 {
        self.next_id += 1;
        self.next_id
    }

    fn update_next_id(&mut self) {
        let max_isp_id = self.isps.iter().map(|isp| isp.id).max().unwrap_or(0);
        let max_website_id = self.websites.iter().map(|website| website.id).max().unwrap_or(0);
        let max_gameserver_id = self.game_servers.iter().map(|gs| gs.id).max().unwrap_or(0);
        self.next_id = max_isp_id.max(max_website_id).max(max_gameserver_id);
    }
}

#[derive(Clone)]
pub struct JsonStore {
    path: PathBuf,
}

impl JsonStore {
    pub fn new(path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create empty file if it doesn't exist
        if !path.exists() {
            let db = Database::default();
            let content = serde_json::to_string_pretty(&db)?;
            fs::write(&path, content)?;
        }

        Ok(Self { path })
    }

    pub async fn load(&self) -> Result<Database> {
        let path = self.path.clone();
        let content = tokio::fs::read_to_string(path).await?;
        let mut db: Database = match serde_json::from_str(&content) {
            Ok(db) => db,
            Err(e) => {
                // If deserialization fails (e.g., missing fields), try to preserve ISPs
                out::warning("db", &format!("Database deserialization error: {}. Attempting recovery...", e));
                let mut db = Database::default();
                // Try to extract ISPs and other data from the partial JSON
                if let Ok(partial) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(isps_array) = partial.get("isps").and_then(|v| v.as_array()) {
                        for isp_value in isps_array {
                            if let Ok(isp) = serde_json::from_value::<crate::models::Isp>(isp_value.clone()) {
                                db.isps.push(isp);
                            }
                        }
                    }
                    if let Some(websites_array) = partial.get("websites").and_then(|v| v.as_array()) {
                        for website_value in websites_array {
                            if let Ok(website) = serde_json::from_value::<crate::models::Website>(website_value.clone()) {
                                db.websites.push(website);
                            }
                        }
                    }
                    if let Some(gs_array) = partial.get("game_servers").and_then(|v| v.as_array()) {
                        for gs_value in gs_array {
                            if let Ok(gs) = serde_json::from_value::<crate::models::GameServer>(gs_value.clone()) {
                                db.game_servers.push(gs);
                            }
                        }
                    }
                }
                db
            }
        };
        db.update_next_id();
        Ok(db)
    }

    pub async fn save(&self, db: &Database) -> Result<()> {
        let path = self.path.clone();
        let content = serde_json::to_string_pretty(db)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    pub async fn read(&self) -> Result<Database> {
        self.load().await
    }

    pub async fn write<F, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut(&mut Database) -> Result<T>,
    {
        let mut db = self.load().await?;
        let result = f(&mut db)?;
        self.save(&db).await?;
        Ok(result)
    }
}

pub fn get_database_path() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                parent.to_path_buf()
            } else {
                PathBuf::from(".")
            }
        } else {
            PathBuf::from(".")
        }
    });
    
    Ok(current_dir.join("net_sentinel.json"))
}

pub async fn init_db() -> Result<JsonStore> {
    let db_path = get_database_path()?;
    out::info("db", &format!("Using JSON database at: {}", db_path.display()));
    let store = JsonStore::new(db_path)?;
    out::ok("db", "Database initialized successfully");
    Ok(store)
}
