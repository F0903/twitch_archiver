use serde::{Deserialize, Serialize};
use std::fs;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const SETTINGS_PATH: &str = "./settings.json";

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub auth_token: Option<String>,
}

fn get_or_create_settings() -> Result<Settings> {
    let settings = match fs::read(SETTINGS_PATH) {
        Ok(x) => serde_json::from_slice::<Settings>(&x)?,
        _ => Settings { auth_token: None },
    };
    Ok(settings)
}

fn write_settings(settings: Settings) -> Result<()> {
    serde_json::to_writer(fs::File::create(SETTINGS_PATH)?, &settings)?;
    Ok(())
}

pub fn set(modifier: impl Fn(&mut Settings)) -> Result<()> {
    let mut settings = get_or_create_settings()?;
    modifier(&mut settings);
    write_settings(settings)?;
    Ok(())
}

pub fn get() -> Result<Settings> {
    get_or_create_settings()
}
