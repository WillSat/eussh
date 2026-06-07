use serde::{Deserialize, Serialize};
use crate::models::connection::ConnectionProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub connections: Vec<ConnectionProfile>,
    #[serde(default)]
    pub settings: AppSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            language: String::new(),
            connections: Vec::new(),
            settings: AppSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_font_size")]
    pub font_size: u16,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    #[serde(default = "default_scrollback")]
    pub scrollback: u32,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u16,
    #[serde(default = "default_color_preset")]
    pub terminal_color_preset: String,
    #[serde(default = "default_monitor_refresh")]
    pub monitor_refresh_secs: u32,
    #[serde(default = "default_ping_interval")]
    pub ping_interval_secs: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            font_family: default_font_family(),
            cursor_style: default_cursor_style(),
            scrollback: default_scrollback(),
            sidebar_width: default_sidebar_width(),
            terminal_color_preset: default_color_preset(),
            monitor_refresh_secs: default_monitor_refresh(),
            ping_interval_secs: default_ping_interval(),
        }
    }
}

fn default_font_size() -> u16 { 14 }
fn default_font_family() -> String { "monospace".into() }
fn default_cursor_style() -> String { "bar".into() }
fn default_scrollback() -> u32 { 10000 }
fn default_sidebar_width() -> u16 { 260 }
fn default_color_preset() -> String { "default-dark".into() }
fn default_monitor_refresh() -> u32 { 2 }
fn default_ping_interval() -> u32 { 15 }
