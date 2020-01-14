use serde::{Deserialize, Serialize};
use std::fs;
use std::{error::Error, path::Path, collections::HashMap};
use termion::color;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub indent_string: String,
    pub day_change_time: i32,
    pub repos: Vec<String>,
    pub emails: Vec<String>,
    pub foreground_colors: HashMap<String, String>,
    pub background_colors: HashMap<String, String>,
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
        let foreground_colors = [
            ("default", "default"),
            ("add", "green"),
            ("delete", "red"),
            ("modify", "blue"),
            ("highlight", "black"),
        ].iter().cloned().map(|t| (t.0.to_string(), t.1.to_string())).collect();

        let background_colors = [
            ("default", "default"),
            ("highlight", "yellow"),
        ].iter().cloned().map(|t| (t.0.to_string(), t.1.to_string())).collect();

        Config {
            indent_string: String::from("    "),
            day_change_time: 500,
            repos: Vec::new(),
            emails: Vec::new(),
            foreground_colors,
            background_colors,
        }
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let text = serde_json::to_string(&self)?;
        fs::write(path, text)?;
        Ok(())
    }


    pub fn get_color_settings(&self) -> Result<Colors, Box<dyn Error>> {
        let mut colors = Colors {
            fg: HashMap::new(),
            bg: HashMap::new(),
        };
        for (setting, color_string) in &self.foreground_colors {
            colors.fg.insert(setting.clone(), get_color_escape(color_string, true)?);
        }
        for (setting, color_string) in &self.background_colors {
            colors.bg.insert(setting.clone(), get_color_escape(color_string, false)?);
        }
        Ok(colors)
    }
}

pub struct Colors {
    fg: HashMap<String, String>,
    bg: HashMap<String, String>
}

impl Colors {
    fn fg(&self, setting: &str) -> &str {
        match self.fg.get(setting) {
            Some(s) => s, 
            None => color::Reset.fg_str(),
        }
    }

    fn bg(&self, setting: &str) -> &str {
        match self.fg.get(setting) {
            Some(s) => s, 
            None => color::Reset.bg_str(),
        }
    }
}

fn get_color_escape(str_color: &str, is_fg: bool) -> Result<String, Box<dyn Error>> {
    let mut found = true;
    let ansi;
    if is_fg {
        ansi = match str_color {
            "default" => color::Reset.fg_str().to_string(),
            "black" => color::Black.fg_str().to_string(),
            "blue" => color::Blue.fg_str().to_string(),
            "cyan" => color::Cyan.fg_str().to_string(),
            "green" => color::Green.fg_str().to_string(),
            "magenta" => color::Magenta.fg_str().to_string(),
            "red" => color::Red.fg_str().to_string(),
            "white" => color::White.fg_str().to_string(),
            "yellow" => color::Yellow.fg_str().to_string(),
            "light black" => color::LightBlack.fg_str().to_string(),
            "light blue" => color::LightBlue.fg_str().to_string(),
            "light cyan" => color::LightCyan.fg_str().to_string(),
            "light green" => color::LightGreen.fg_str().to_string(),
            "light magenta" => color::LightMagenta.fg_str().to_string(),
            "light red" => color::LightRed.fg_str().to_string(),
            "light white" => color::White.fg_str().to_string(),
            "light yellow" => color::LightYellow.fg_str().to_string(),
            _ => { 
                found = false;
                "".to_owned()
            }
        }
    } else {
        ansi = match str_color {
            "default" => color::Reset.bg_str().to_string(),
            "black" => color::Black.bg_str().to_string(),
            "blue" => color::Blue.bg_str().to_string(),
            "cyan" => color::Cyan.bg_str().to_string(),
            "green" => color::Green.bg_str().to_string(),
            "magenta" => color::Magenta.bg_str().to_string(),
            "red" => color::Red.bg_str().to_string(),
            "white" => color::White.bg_str().to_string(),
            "yellow" => color::Yellow.bg_str().to_string(),
            "light black" => color::LightBlack.bg_str().to_string(),
            "light blue" => color::LightBlue.bg_str().to_string(),
            "light cyan" => color::LightCyan.bg_str().to_string(),
            "light green" => color::LightGreen.bg_str().to_string(),
            "light magenta" => color::LightMagenta.bg_str().to_string(),
            "light red" => color::LightRed.bg_str().to_string(),
            "light white" => color::White.bg_str().to_string(),
            "light yellow" => color::LightYellow.bg_str().to_string(),
            _ => { 
                found = false;
                "".to_owned()
            }
        }
    }
    if found {
        return Ok(ansi);
    }
    let str_nums: Vec<&str> = str_color.split_ascii_whitespace().collect();
    let mut color_values: Vec<u8> = Vec::new();
    for str_num in str_nums {
        color_values.push(str_num.parse()?)
    }

    if color_values.len() == 3 {
        let rgb_color =  color::Rgb(color_values[0], color_values[1], color_values[2]);
        if is_fg {
            Ok(rgb_color.fg_string())
        } else {
            Ok(rgb_color.bg_string())
        }
    } else {
        Err(Box::new(crate::Error::new("Color Config must be r g b")))
    }
}

pub fn get_base_path() -> Result<String, Box<dyn Error>> {
    Ok(format!("{}/.gitintegratedjournal/", std::env::var("HOME")?))
}
