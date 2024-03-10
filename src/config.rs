use color_eyre::{eyre::WrapErr, Result};
use mastodon_async::{data::Data, helpers::toml};
use tracing::info;

#[derive(Debug, Clone)]
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
    /// e.g. ~/.config/toot-rs/config.toml
    pub fn load() -> Result<Self> {
        let xdg = xdg::BaseDirectories::with_prefix("toot-rs")?;
        let config_file = xdg.get_config_file("config.toml");
        let data = toml::from_file(&config_file).with_context(|| {
            format!("unable to read config file from {}", &config_file.display())
        })?;
        info!("Loaded config file from {}", &config_file.display());
        Ok(Self { data })
    }

    /// Saves the config file to the XDG config directory
    /// e.g. ~/.config/toot-rs/config.toml
    /// If the file already exists, it will be overwritten
    /// If the directory does not exist, it will be created
    pub fn save(&self) -> Result<()> {
        let xdg = xdg::BaseDirectories::with_prefix("toot-rs")?;
        let config_file = xdg.place_config_file("config.toml")?;
        toml::to_file(&self.data, &config_file).with_context(|| {
            format!("unable to write config file to {}", &config_file.display())
        })?;
        info!("Saved config file to {}", &config_file.display());
        Ok(())
    }
}
