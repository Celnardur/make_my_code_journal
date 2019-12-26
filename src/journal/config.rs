use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub indent_string: String,
    pub day_change_time: i32,
    pub repos: Vec<String>, 
    pub emails: Vec<String>,
}

impl Config {
    pub fn new() -> Result<Config, Box<dyn Error>> {
        // command line argumtents don't do anything currently 
        //let args: Vec<String> = env::args().collect();
        
        let mut path = get_base_path()?;
        path.push_str("settings.json");
        if !Path::new(&path).exists() {
            //TODO: Throw error for now make setup dialouge latter
            return Err(Box::new(super::Error::new("No Settings Found")));
        }

        let file_str = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&file_str)?)
    }
}

pub fn get_base_path() -> Result<String, Box<dyn Error>> {
    Ok(format!("{}/.gitintegratedjournal/", std::env::var("HOME")?))
}
