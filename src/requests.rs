use serde::Deserialize;
use reqwest::blocking::Client;

use crate::state::{ButtonType, Config, UserDataSummary};

#[derive(Deserialize)]
struct Record {
    score: f64,
    #[serde(rename = "maxCombo")]
    max_combo: bool,
}

#[derive(Deserialize)]
struct RawUserData {
    success: bool,
    count: i32,
    records: Vec<Record>,
}

#[derive(Deserialize)]
struct RawUserTier {
    success: bool,
    #[serde(rename = "tierPoint")]
    tier_point: f64,
}

#[derive(Deserialize)]
struct RawUserDjClass {
    success: bool,
    #[serde(rename = "djClass")]
    dj_class: String,
}

fn fetch_raw_user_data(client: &Client,username: &str, button: i8) -> Result<RawUserData, reqwest::Error> {
    let url = format!("https://v-archive.net/api/v2/archive/{}/button/{}", username, button);
    client.get(&url).send()?.json::<RawUserData>()
}

fn fetch_raw_user_tier(client: &Client,username: &str, button: i8) -> Result<RawUserTier, reqwest::Error> {
    let url = format!("https://v-archive.net/api/v2/archive/{}/tier/{}", username, button);
    client.get(&url).send()?.json::<RawUserTier>()
}

fn fetch_raw_user_dj_class(client: &Client, username: &str, button: i8) -> Result<RawUserDjClass, reqwest::Error> {
    let url = format!("https://v-archive.net/api/v2/archive/{}/djClass/{}", username, button);
    client.get(&url).send()?.json::<RawUserDjClass>()
}

pub fn get_data(username: &str, button: i8) -> Option<UserDataSummary> {
    let client = Client::new();
    let raw = fetch_raw_user_data(&client, username, button).ok()?;
    if !raw.success {
        return None;
    }

    let raw_tier = fetch_raw_user_tier(&client, username, button).ok()?;
    if !raw_tier.success {
        return None;
    }

    let raw_class = fetch_raw_user_dj_class(&client, username, button).ok()?;
    if !raw_class.success { 
        return None;
    }

    let mut total_score: i64 = 0; 
    let mut total_perfects = 0;
    let mut total_max_combos = 0;
    let records_len = raw.records.len();

    for record in &raw.records {
        if (record.score - 100.0).abs() < f64::EPSILON {
            total_perfects += 1;
        }
        
        total_score += (record.score * 100.0).floor() as i64;

        if record.max_combo {
            total_max_combos += 1;
        }
    }

    let avg_rating = if records_len > 0 {
        ((total_score as f64 / records_len as f64).round()) / 100.0
    } else {
        0.0
    };

    Some(UserDataSummary {
        clears: raw.count,
        perfects: total_perfects,
        max_combos: total_max_combos,
        avg_rating,
        tier_point: raw_tier.tier_point,
        dj_class: raw_class.dj_class,
    })
}

pub fn update_data(config: &Config, data: &UserDataSummary, button: &ButtonType, username: &str) -> String {
    let url = format!(
        "https://discord.com/api/v9/applications/{}/users/{}/identities/0/profile",
        config.app, config.user
    );

    // Build the dynamic payload using serde_json::json!
    let payload = serde_json::json!({
        "data": {
            "dynamic": [
                {
                    "type": 1,
                    "name": "clears",
                    "value": format!("{}개", data.clears)
                },
                {
                    "type": 1,
                    "name": "perfect",
                    "value": format!("{}개", data.perfects)
                },
                {
                    "type": 1,
                    "name": "maxcombo",
                    "value": format!("{}개", data.max_combos)
                },
                {
                    "type": 1,
                    "name": "avgrate",
                    "value": format!("{}%", data.avg_rating)
                },
                {
                    "type": 1,
                    "name": "tierpoints",
                    "value": data.tier_point.to_string()
                },
                {
                    "type": 1,
                    "name": "djclass",
                    "value": data.dj_class
                },
                {
                    "type": 1,
                    "name": "button",
                    "value": button.to_string()
                },
                {
                    "type": 1,
                    "name": "username",
                    "value": username
                },
                {
                    "type": 3,
                    "name": "icon",
                    "value": {
                        "url": "https://raw.githubusercontent.com/dytroc/assets/refs/heads/main/djmax.webp"
                    }
                }
            ]
        }
    });

    let client = Client::new();
    let response = client
        .patch(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {}", config.token))
        .json(&payload)
        .send();

    match response {
        Ok(res) if res.status().is_success() => {
            format!("업데이트에 성공했습니다!")
        }
        Ok(res) => {
            format!("업데이트에 실패했습니다... HTTP 상태 코드: {}", res.status())
        }
        Err(_) => {
            format!("업데이트에 실패했습니다...")
        }
    }
}