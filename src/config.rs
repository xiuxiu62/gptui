use crate::DynResult;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    api_key: String,
    conversation_file: Option<PathBuf>,
    ai_name: Option<String>,
}

impl Config {
    pub fn generate() -> DynResult<Self> {
        // Prompt for api key
        let mut api_key = "".to_owned();
        print!("{}: ", "Enter your api key".purple());
        crate::io::flush()?;
        io::stdin().read_line(&mut api_key).unwrap();

        // Create config
        let config = Self {
            api_key: api_key.trim_end().to_owned(),
            conversation_file: None,
            ai_name: None,
        };

        // Open file or create if it doesn't exist
        let path = default_path();
        let mut file = match File::create(&path).ok() {
            Some(file) => file,
            None => {
                fs::create_dir_all(path.parent().unwrap())?;
                File::create(path)?
            }
        };

        // Serialize and write config
        let contents = serde_json::to_string_pretty(&config)?;
        write!(file, "{contents}\n")?;

        Ok(config)
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn ai_name(&self) -> &str {
        self.ai_name
            .as_ref()
            .map(|name| name.as_str())
            .unwrap_or("gpt")
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = io::Error;

    fn try_from(path: PathBuf) -> io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let inner = serde_json::from_str(&data)?;

        Ok(inner)
    }
}

#[inline]
pub fn default_path() -> PathBuf {
    PathBuf::from(format!(
        "/home/{}/.config/gptui/config.json",
        whoami::username()
    ))
}
