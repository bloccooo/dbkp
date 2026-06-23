use anyhow::{Result, anyhow};
use dbkp_core::{databases::DatabaseConfig, storage::provider::StorageConfig};
use serde::{Deserialize, Serialize};
use std::env;
use std::{borrow::Borrow, fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    pub config_path: PathBuf,
    database_configs: Vec<DatabaseConfig>,
    storage_configs: Vec<StorageConfig>,
}

impl Configs {
    pub fn load() -> Result<Self> {
        let config_path = match env::var("DBKP_CONFIG_PATH") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                let config_dir = dirs::config_dir()
                    .ok_or_else(|| anyhow!("Could not determine config directory"))?
                    .join("dbkp");

                config_dir.join("app_storage.json")
            }
        };

        let config_dir = match config_path.parent() {
            Some(parent) => parent,
            None => {
                return Err(anyhow!(
                    "Unable to get parent directory for the config path"
                ));
            }
        };

        if !config_path.exists() {
            fs::create_dir_all(&config_dir)?;

            let app_storage = Self {
                config_path: config_path.clone(),
                database_configs: vec![],
                storage_configs: vec![],
            };

            let content = serde_json::to_string_pretty(&app_storage)?;
            fs::write(&config_path, content)?;
        }

        let content = fs::read_to_string(&config_path)?;
        let mut app_storage: Self = serde_json::from_str(&content)?;
        if app_storage.config_path != config_path {
            app_storage.config_path = config_path;
            app_storage.save()?;
        }

        return Ok(app_storage);
    }

    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn get_database_configs(&self) -> Vec<DatabaseConfig> {
        return self.database_configs.clone();
    }

    pub fn add_database_config<D>(&mut self, database_config: D) -> Result<()>
    where
        D: Borrow<DatabaseConfig>,
    {
        self.database_configs.push(database_config.borrow().clone());
        self.save()?;

        Ok(())
    }

    pub fn get_storage_configs(&self) -> Vec<StorageConfig> {
        return self.storage_configs.clone();
    }

    pub fn add_storage_config<S>(&mut self, storage_config: S) -> Result<()>
    where
        S: Borrow<StorageConfig>,
    {
        self.storage_configs.push(storage_config.borrow().clone());
        self.save()?;

        Ok(())
    }
}
