use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct MainAppState {
    pub config: Config,
    pub username: String,
    pub button: ButtonType,
    pub summary: Option<UserDataSummary>
}

#[derive(Default, PartialEq, Copy, Clone, strum::EnumIter, Serialize, Deserialize)]
pub enum ButtonType {
    #[default] FOUR, FIVE, SIX, EIGHT
}

impl Display for ButtonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FOUR => write!(f, "4B"),
            Self::FIVE => write!(f, "5B"),
            Self::SIX => write!(f, "6B"),
            Self::EIGHT => write!(f, "8B")
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct MainAppRawState {
    pub config: String
}

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub app: String,
    pub user: String,
    pub token: String
}

#[derive(Default, Deserialize, Serialize)]
pub struct UserDataSummary {
    pub clears: i32,
    pub perfects: i32,
    pub max_combos: i32,
    pub avg_rating: f64,
    pub tier_point: f64,
    pub dj_class: String,
}