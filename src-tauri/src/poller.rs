use tauri::Emitter;
use crate::{engine, game_state::GameSnapshot, ai_advisor::AiAdvisor};

const POLL_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL_SECS: u64 = 3;

pub async fn start_polling(app_handle: tauri::AppHandle, api_key: String) {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .expect("Failed to build HTTP client");

    let kb = engine::load_knowledge_base();
    let mut state = engine::EngineState::default();
    let mut ai = AiAdvisor::new(api_key);

    loop {
        match fetch_snapshot(&client).await {
            Ok(snapshot) => {
                let mut advice = engine::generate_advice(&snapshot, &mut state, &kb);

                let my_name = state.summoner_name.as_deref().unwrap_or("").to_string();
                ai.maybe_refresh(&snapshot, &my_name);

                // Overlay AI-generated tips when available (falls back to rule-based)
                if let Some(ai_advice) = ai.get() {
                    advice.team_fight_tip = ai_advice.team_fight_tip;
                    advice.support_tip = ai_advice.support_tip;
                    if !ai_advice.lane_tips.is_empty() {
                        advice.lane_tip = Some(ai_advice.lane_tips);
                    }
                }

                let _ = app_handle.emit("game-advice", &advice);
            }
            Err(_) => {
                state.reset();
                let _ = app_handle.emit("game-idle", ());
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn fetch_snapshot(client: &reqwest::Client) -> Result<GameSnapshot, reqwest::Error> {
    let response = client.get(POLL_URL).send().await?;
    let snapshot = response.json::<GameSnapshot>().await?;
    Ok(snapshot)
}
