use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GameSnapshot {
    #[serde(rename = "activePlayer")]
    pub active_player: ActivePlayer,
    #[serde(rename = "allPlayers")]
    pub all_players: Vec<PlayerData>,
    pub events: Events,
    #[serde(rename = "gameData")]
    pub game_data: GameData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ActivePlayer {
    #[serde(rename = "currentGold")]
    pub current_gold: f32,
    #[serde(rename = "summonerName")]
    pub summoner_name: String,
    pub level: u8,
    #[serde(rename = "championStats")]
    pub champion_stats: ChampionStats,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChampionStats {
    #[serde(rename = "currentHealth")]
    pub current_health: f32,
    #[serde(rename = "maxHealth")]
    pub max_health: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerData {
    #[serde(rename = "championName")]
    pub champion_name: String,
    #[serde(rename = "summonerName")]
    pub summoner_name: String,
    pub team: String,
    pub items: Vec<ItemData>,
    pub level: u8,
    pub position: String,
    pub scores: Scores,
    #[serde(rename = "isDead")]
    pub is_dead: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ItemData {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "itemID")]
    pub item_id: u32,
    pub slot: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Scores {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    #[serde(rename = "creepScore")]
    pub creep_score: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Events {
    #[serde(rename = "Events")]
    pub events: Vec<GameEvent>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameEvent {
    #[serde(rename = "EventName")]
    pub event_name: String,
    #[serde(rename = "EventTime")]
    pub event_time: f32,
    #[serde(rename = "EventID")]
    pub event_id: u32,
    #[serde(rename = "DragonType")]
    pub dragon_type: Option<String>,
    #[serde(rename = "KillerName")]
    pub killer_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameData {
    #[serde(rename = "gameTime")]
    pub game_time: f32,
    #[serde(rename = "gameMode")]
    pub game_mode: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot_json() -> &'static str {
        r#"{
          "activePlayer": {
            "currentGold": 1250.0,
            "summonerName": "TestPlayer",
            "level": 8,
            "championStats": { "currentHealth": 900.0, "maxHealth": 1400.0 }
          },
          "allPlayers": [
            {
              "championName": "Twitch",
              "summonerName": "TestPlayer",
              "team": "ORDER",
              "items": [
                { "displayName": "Doran's Blade", "itemID": 1055, "slot": 0 }
              ],
              "level": 8,
              "position": "BOTTOM",
              "scores": { "kills": 2, "deaths": 1, "assists": 3, "creepScore": 85 },
              "isDead": false
            },
            {
              "championName": "Soraka",
              "summonerName": "Ally",
              "team": "ORDER",
              "items": [],
              "level": 7,
              "position": "SUPPORT",
              "scores": { "kills": 0, "deaths": 0, "assists": 5, "creepScore": 10 },
              "isDead": false
            },
            {
              "championName": "Caitlyn",
              "summonerName": "Enemy1",
              "team": "CHAOS",
              "items": [],
              "level": 8,
              "position": "BOTTOM",
              "scores": { "kills": 1, "deaths": 2, "assists": 0, "creepScore": 90 },
              "isDead": false
            }
          ],
          "events": {
            "Events": [
              { "EventID": 1, "EventName": "GameStart", "EventTime": 0.0 },
              { "EventID": 3, "EventName": "FirstBlood", "EventTime": 92.5, "KillerName": "TestPlayer" }
            ]
          },
          "gameData": {
            "gameTime": 865.2,
            "gameMode": "CLASSIC"
          }
        }"#
    }

    #[test]
    fn test_parse_full_snapshot() {
        let snapshot: GameSnapshot = serde_json::from_str(sample_snapshot_json()).unwrap();
        assert_eq!(snapshot.active_player.summoner_name, "TestPlayer");
        assert_eq!(snapshot.active_player.current_gold, 1250.0);
        assert_eq!(snapshot.active_player.level, 8);
        assert_eq!(snapshot.all_players.len(), 3);
        assert_eq!(snapshot.game_data.game_time, 865.2);
    }

    #[test]
    fn test_parse_player_items() {
        let snapshot: GameSnapshot = serde_json::from_str(sample_snapshot_json()).unwrap();
        let twitch = &snapshot.all_players[0];
        assert_eq!(twitch.champion_name, "Twitch");
        assert_eq!(twitch.items.len(), 1);
        assert_eq!(twitch.items[0].item_id, 1055);
        assert_eq!(twitch.items[0].display_name, "Doran's Blade");
    }

    #[test]
    fn test_parse_events() {
        let snapshot: GameSnapshot = serde_json::from_str(sample_snapshot_json()).unwrap();
        assert_eq!(snapshot.events.events.len(), 2);
        let first_blood = &snapshot.events.events[1];
        assert_eq!(first_blood.event_name, "FirstBlood");
        assert_eq!(first_blood.killer_name, Some("TestPlayer".to_string()));
    }

    #[test]
    fn test_dragon_type_is_optional() {
        let snapshot: GameSnapshot = serde_json::from_str(sample_snapshot_json()).unwrap();
        // GameStart event has no DragonType
        assert_eq!(snapshot.events.events[0].dragon_type, None);
    }
}
