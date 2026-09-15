use crate::config::Error;
use crate::config::app::AppConfig;

pub fn read_config(config_path: &String)
    -> Result<AppConfig, Error>
{
    let content = std::fs::read_to_string(config_path)?;
    let config = serde_yaml_ng::from_str::<AppConfig>(&content)?;
    Ok(config)
}
