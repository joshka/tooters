use anyhow::{Context, Result};
use mastodon_async::{data::Data, helpers::toml};

#[derive(Debug)]
pub struct Config {
    pub data: Data,
}

impl From<Data> for Config {
    fn from(data: Data) -> Self {
        Self { data }
    }
}

impl Config {
    /// Loads the config file from the XDG config directory
    /// e.g. ~/.config/tooters/config.toml
    pub fn load() -> Result<Self> {
        let xdg = xdg::BaseDirectories::with_prefix("tooters")?;
        let config_file = xdg.get_config_file("config.toml");
        let data = toml::from_file(&config_file).with_context(|| {
            format!(
                "Unable to read config file from: {}",
                &config_file.to_string_lossy()
            )
        })?;
        Ok(Self { data })
    }

    /// Saves the config file to the XDG config directory
    /// e.g. ~/.config/tooters/config.toml
    /// If the file already exists, it will be overwritten
    /// If the directory does not exist, it will be created
    pub fn save(&self) -> Result<String> {
        let xdg = xdg::BaseDirectories::with_prefix("tooters")?;
        let config_file = xdg.place_config_file("config.toml")?;
        toml::to_file(&self.data, &config_file).with_context(|| {
            format!(
                "Unable to write config file to: {}",
                &config_file.to_string_lossy()
            )
        })?;
        Ok(config_file.to_string_lossy().to_string())
    }
}
