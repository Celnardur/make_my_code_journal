use serde::{Deserialize, Serialize};
use std::fs;
use std::{error::Error, path::Path};
use termion::color::AnsiValue;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub indent_string: String,
    pub day_change_time: i32,
    pub repos: Vec<String>,
    pub emails: Vec<String>,
    pub colors: RGBColorSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RGBColorSettings {
    pub fg_default: Color,
    pub bg_default: Color,
    pub fg_highlight: Color,
    pub bg_highlight: Color,
    pub fg_add: Color,
    pub bg_add: Color,
    pub hl_add: Color,
    pub fg_delete: Color,
    pub bg_delete: Color,
    pub hl_delete: Color,
}

pub struct ColorSettings {
    pub fg_default: String,
    pub bg_default: String,
    pub fg_highlight: String,
    pub bg_highlight: String,
    pub fg_add: String,
    pub bg_add: String,
    pub hl_add: String,
    pub fg_delete: String,
    pub bg_delete: String,
    pub hl_delete: String,
}

impl ColorSettings {
    pub fn default() -> ColorSettings {
        ColorSettings {
            fg_default: String::new(),
            bg_default: String::new(),
            fg_highlight: String::new(),
            bg_highlight: String::new(),
            fg_add: String::new(),
            bg_add: String::new(),
            hl_add: String::new(),
            fg_delete: String::new(),
            bg_delete: String::new(),
            hl_delete: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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

    pub fn default() -> Config {
        Config {
            indent_string: String::from("    "),
            day_change_time: 500,
            repos: Vec::new(),
            emails: Vec::new(),
            colors: RGBColorSettings {
                fg_default: Color { r: 5, g: 5, b: 5 },
                bg_default: Color { r: 0, g: 0, b: 0 },
                fg_highlight: Color { r: 0, g: 0, b: 0 },
                bg_highlight: Color { r: 5, g: 5, b: 1 },
                fg_add: Color { r: 0, g: 5, b: 0 },
                bg_add: Color { r: 0, g: 0, b: 0 },
                hl_add: Color { r: 5, g: 5, b: 5 },
                fg_delete: Color { r: 5, g: 0, b: 0 },
                bg_delete: Color { r: 0, g: 0, b: 0 },
                hl_delete: Color { r: 5, g: 5, b: 5 },
            },
        }
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let text = serde_json::to_string(&self)?;
        fs::write(path, text)?;
        Ok(())
    }

    pub fn get_color_settings(&self) -> Result<ColorSettings, Box<dyn Error>> {
        Ok(ColorSettings {
            fg_default: {
                let c = self.colors.fg_default;
                AnsiValue::rgb(c.r, c.g, c.b).fg_string()
            },
            bg_default: {
                let c = self.colors.bg_default;
                AnsiValue::rgb(c.r, c.g, c.b).bg_string()
            },
            fg_highlight: {
                let c = self.colors.fg_highlight;
                AnsiValue::rgb(c.r, c.g, c.b).fg_string()
            },
            bg_highlight: {
                let c = self.colors.bg_highlight;
                AnsiValue::rgb(c.r, c.g, c.b).bg_string()
            },
            fg_add: {
                let c = self.colors.fg_add;
                AnsiValue::rgb(c.r, c.g, c.b).fg_string()
            },
            bg_add: {
                let c = self.colors.bg_add;
                AnsiValue::rgb(c.r, c.g, c.b).bg_string()
            },
            hl_add: {
                let c = self.colors.hl_add;
                AnsiValue::rgb(c.r, c.g, c.b).fg_string()
            },
            fg_delete: {
                let c = self.colors.fg_delete;
                AnsiValue::rgb(c.r, c.g, c.b).fg_string()
            },
            bg_delete: {
                let c = self.colors.bg_delete;
                AnsiValue::rgb(c.r, c.g, c.b).bg_string()
            },
            hl_delete: {
                let c = self.colors.hl_delete;
                AnsiValue::rgb(c.r, c.g, c.b).fg_string()
            },
        })
    }
}

pub fn get_base_path() -> Result<String, Box<dyn Error>> {
    Ok(format!("{}/.gitintegratedjournal/", std::env::var("HOME")?))
}
