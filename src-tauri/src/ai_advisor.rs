use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use crate::game_state::GameSnapshot;

// ── AI advice output ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAdvice {
    pub team_fight_tip: String,
    pub support_tip: Option<String>,
    pub lane_tips: Vec<String>,
}

// ── State change detection ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
struct StateKey {
    total_kills: u32,
    total_items: usize,
    event_count: usize,
    game_minute: u32,
}

fn compute_state_key(snapshot: &GameSnapshot) -> StateKey {
    let total_kills = snapshot.all_players.iter().map(|p| p.scores.kills).sum();
    let total_items = snapshot.all_players.iter().map(|p| p.items.len()).sum();
    let event_count = snapshot.events.events.len();
    let game_minute = (snapshot.game_data.game_time / 60.0) as u32;
    StateKey { total_kills, total_items, event_count, game_minute }
}

// ── Advisor ────────────────────────────────────────────────────────────────

pub struct AiAdvisor {
    current: Arc<RwLock<Option<AiAdvice>>>,
    last_state: Option<StateKey>,
    last_refresh_time: f32,
    api_key: String,
}

impl AiAdvisor {
    pub fn new(api_key: String) -> Self {
        Self {
            current: Arc::new(RwLock::new(None)),
            last_state: None,
            last_refresh_time: -999.0,
            api_key,
        }
    }

    pub fn get(&self) -> Option<AiAdvice> {
        self.current.read().ok().and_then(|g| g.clone())
    }

    /// Called each poll cycle. Spawns a background AI refresh if state has changed
    /// or 60 seconds have elapsed since the last refresh.
    pub fn maybe_refresh(&mut self, snapshot: &GameSnapshot, my_name: &str) {
        if self.api_key.is_empty() {
            return;
        }

        let key = compute_state_key(snapshot);
        let game_time = snapshot.game_data.game_time;
        let state_changed = self.last_state.as_ref() != Some(&key);
        let heartbeat_elapsed = (game_time - self.last_refresh_time) > 60.0;

        if state_changed || heartbeat_elapsed {
            self.last_state = Some(key);
            self.last_refresh_time = game_time;

            let api_key = self.api_key.clone();
            let current = Arc::clone(&self.current);
            let snapshot = snapshot.clone();
            let my_name = my_name.to_string();

            tokio::spawn(async move {
                match call_openrouter(&api_key, &snapshot, &my_name).await {
                    Ok(advice) => {
                        if let Ok(mut lock) = current.write() {
                            *lock = Some(advice);
                        }
                    }
                    Err(e) => {
                        eprintln!("[ai_advisor] OpenRouter error: {}", e);
                    }
                }
            });
        }
    }
}

// ── OpenRouter call ────────────────────────────────────────────────────────

async fn call_openrouter(
    api_key: &str,
    snapshot: &GameSnapshot,
    my_name: &str,
) -> Result<AiAdvice, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = build_prompt(snapshot, my_name);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let body = serde_json::json!({
        "model": "openai/gpt-oss-120b:free",
        "messages": [
            {
                "role": "system",
                "content": "You are a League of Legends advisor for Twitch ADC. Respond ONLY with valid JSON matching this exact schema: {\"team_fight_tip\": string, \"support_tip\": string | null, \"lane_tips\": string[]}. Each tip must be under 20 words. No markdown, no explanation, just the JSON object."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 200
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let resp_json: serde_json::Value = response.json().await?;
    let content = resp_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in OpenRouter response")?
        .to_string();

    let cleaned = extract_json(&content);
    let advice: AiAdvice = serde_json::from_str(&cleaned)?;
    Ok(advice)
}

fn build_prompt(snapshot: &GameSnapshot, my_name: &str) -> String {
    let game_min = (snapshot.game_data.game_time / 60.0) as u32;

    let me = snapshot.all_players.iter().find(|p| p.summoner_name == my_name);
    let my_team = me.map(|p| p.team.as_str()).unwrap_or("ORDER");

    let my_items: Vec<&str> = me
        .map(|p| p.items.iter().map(|i| i.display_name.as_str()).collect())
        .unwrap_or_default();

    let my_kda = me
        .map(|p| format!("{}/{}/{}", p.scores.kills, p.scores.deaths, p.scores.assists))
        .unwrap_or_else(|| "0/0/0".to_string());

    let support = snapshot.all_players.iter()
        .find(|p| p.team == my_team && p.position == "SUPPORT" && p.summoner_name != my_name)
        .map(|p| p.champion_name.as_str())
        .unwrap_or("unknown");

    let enemies: Vec<&str> = snapshot.all_players.iter()
        .filter(|p| p.team != my_team)
        .map(|p| p.champion_name.as_str())
        .collect();

    let recent_events: Vec<&str> = snapshot.events.events.iter()
        .rev()
        .take(3)
        .map(|e| e.event_name.as_str())
        .collect();

    format!(
        "Game: {}min | Champion: Twitch ADC | Items: {} | KDA: {} | Support: {} | Enemies: {} | Recent events: {}",
        game_min,
        if my_items.is_empty() { "none".to_string() } else { my_items.join(", ") },
        my_kda,
        support,
        if enemies.is_empty() { "unknown".to_string() } else { enemies.join(", ") },
        if recent_events.is_empty() { "none".to_string() } else { recent_events.join(", ") },
    )
}

/// Strip markdown code fences if the model wraps the JSON in ```json ... ```
fn extract_json(content: &str) -> String {
    let trimmed = content.trim();
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return trimmed[start..=end].to_string();
        }
    }
    trimmed.to_string()
}
