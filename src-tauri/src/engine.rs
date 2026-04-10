use serde::{Deserialize, Serialize};
use crate::game_state::{GameSnapshot, PlayerData};

// ── Knowledge base structs ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ItemEntry {
    pub id: String,
    pub name: String,
    pub cost: u32,
    pub category: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SynergyEntry {
    pub champion: String,
    pub tip: String,
}

#[derive(Debug, Deserialize)]
pub struct MatchupEntry {
    pub name: String,
    pub role: String,
    pub archetype: String,
    pub tip: String,
}

#[derive(Debug)]
pub struct KnowledgeBase {
    pub items: Vec<ItemEntry>,
    pub synergies: Vec<SynergyEntry>,
    pub matchups: Vec<MatchupEntry>,
}

pub fn load_knowledge_base() -> KnowledgeBase {
    let items: Vec<ItemEntry> = serde_json::from_str(include_str!("knowledge/items.json"))
        .expect("Failed to parse items.json");
    let synergies: Vec<SynergyEntry> = serde_json::from_str(include_str!("knowledge/synergies.json"))
        .expect("Failed to parse synergies.json");
    let matchups: Vec<MatchupEntry> = serde_json::from_str(include_str!("knowledge/matchups.json"))
        .expect("Failed to parse matchups.json");
    KnowledgeBase { items, synergies, matchups }
}

// ── Output types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum BuildPath {
    CritLethality,
    PureCrit,
    OnHit,
}

#[derive(Debug, Serialize, Clone)]
pub struct SuggestedItem {
    pub name: String,
    pub reason: String,
    pub priority: u8,
}

#[derive(Debug, Serialize, Clone)]
pub struct GameAdvice {
    pub build_path: BuildPath,
    pub built_items: Vec<String>,
    pub suggested_items: Vec<SuggestedItem>,
    pub first_back_note: Option<String>,
    pub support_tip: Option<String>,
    pub lane_tip: Option<String>,
    pub team_fight_tip: String,
    pub objective_tip: Option<String>,
    pub game_time: f32,
}

// ── Engine state (persists across polls) ──────────────────────────────────

#[derive(Debug)]
pub struct EngineState {
    pub locked_build_path: Option<BuildPath>,
    pub yun_tal_viable: bool,
    pub first_back_done: bool,
    pub peak_gold_before_first_buy: f32,
    pub previous_item_ids: Vec<u32>,
    pub summoner_name: Option<String>,
}

impl Default for EngineState {
    fn default() -> Self {
        EngineState {
            locked_build_path: None,
            yun_tal_viable: true,
            first_back_done: false,
            peak_gold_before_first_buy: 0.0,
            previous_item_ids: Vec::new(),
            summoner_name: None,
        }
    }
}

impl EngineState {
    pub fn reset(&mut self) {
        *self = EngineState::default();
    }
}

// ── Champion lists ────────────────────────────────────────────────────────

const TANK_CHAMPIONS: &[&str] = &[
    "Malphite", "Cho'Gath", "Dr. Mundo", "Ornn", "Maokai", "Sion",
    "Tahm Kench", "Nautilus", "Leona", "Alistar", "Galio", "Rammus",
    "Zac", "Amumu", "Nunu & Willump",
];

const HEALER_CHAMPIONS: &[&str] = &[
    "Soraka", "Yuumi", "Sona", "Nami", "Seraphine",
];

const HARD_CC_CHAMPIONS: &[&str] = &[
    "Malzahar", "Warwick", "Mordekaiser", "Skarner", "Rammus",
];

const ASSASSIN_CHAMPIONS: &[&str] = &[
    "Zed", "Talon", "Kha'Zix", "Rengar", "Fizz", "Katarina",
];

// ── Build path selection ───────────────────────────────────────────────────

pub fn select_build_path(enemies: &[&PlayerData], state: &mut EngineState) -> BuildPath {
    if let Some(path) = &state.locked_build_path {
        return path.clone();
    }
    let tank_count = enemies.iter()
        .filter(|p| TANK_CHAMPIONS.contains(&p.champion_name.as_str()))
        .count();
    let path = if tank_count >= 3 {
        BuildPath::OnHit
    } else {
        BuildPath::CritLethality
    };
    state.locked_build_path = Some(path.clone());
    path
}

// ── First back detection ──────────────────────────────────────────────────

pub fn check_first_back(
    snapshot: &GameSnapshot,
    my_items: &[u32],
    state: &mut EngineState,
) -> Option<String> {
    if state.first_back_done || snapshot.game_data.game_time > 600.0 {
        return None;
    }
    if my_items == state.previous_item_ids.as_slice()
        && snapshot.active_player.current_gold > state.peak_gold_before_first_buy
    {
        state.peak_gold_before_first_buy = snapshot.active_player.current_gold;
    }
    let new_items: Vec<u32> = my_items.iter()
        .filter(|id| !state.previous_item_ids.contains(id))
        .copied()
        .collect();
    if !new_items.is_empty() && !state.previous_item_ids.is_empty() {
        state.first_back_done = true;
        let gold = state.peak_gold_before_first_buy;
        if gold >= 1300.0 {
            state.yun_tal_viable = true;
            return Some(format!(
                "{:.0}g first back — B.F. Sword path: Yun Tal or Collector available",
                gold
            ));
        } else if gold >= 1100.0 {
            state.yun_tal_viable = false;
            return Some(format!(
                "{:.0}g first back — Serrated Dirk recommended, Collector path",
                gold
            ));
        } else {
            state.yun_tal_viable = false;
            return Some(format!(
                "{:.0}g first back — Long Sword + components. Yun Tal not viable this game",
                gold
            ));
        }
    }
    None
}

// ── Situational item scoring ──────────────────────────────────────────────

pub fn score_situational_items(
    enemies: &[&PlayerData],
    my_built: &[String],
    path: &BuildPath,
    my_scores: &crate::game_state::Scores,
) -> Vec<SuggestedItem> {
    let mut suggestions: Vec<SuggestedItem> = Vec::new();

    let ap_count = enemies.iter().filter(|p| is_ap_champion(&p.champion_name)).count();
    let healer_count = enemies.iter().filter(|p| HEALER_CHAMPIONS.contains(&p.champion_name.as_str())).count();
    let enemy_has_healing_items = enemies.iter().any(|p| p.items.iter().any(|i| is_healing_item(&i.display_name)));
    let tank_count = enemies.iter().filter(|p| TANK_CHAMPIONS.contains(&p.champion_name.as_str())).count();
    let assassin_count = enemies.iter().filter(|p| ASSASSIN_CHAMPIONS.contains(&p.champion_name.as_str())).count();
    let hard_cc_count = enemies.iter().filter(|p| HARD_CC_CHAMPIONS.contains(&p.champion_name.as_str())).count();
    let winning = my_scores.kills > my_scores.deaths + 1;

    let already_built = |name: &str| my_built.iter().any(|b| b == name);

    if ap_count >= 3 && !already_built("Wit's End") && !already_built("Maw of Malmortius") {
        let item_name = match path { BuildPath::OnHit => "Wit's End", _ => "Maw of Malmortius" };
        suggestions.push(SuggestedItem {
            name: item_name.to_string(),
            reason: format!("{} AP threats — magic resist", ap_count),
            priority: 4,
        });
    }

    if (healer_count > 0 || enemy_has_healing_items) && !already_built("Mortal Reminder") {
        suggestions.push(SuggestedItem {
            name: "Mortal Reminder".to_string(),
            reason: "Enemy healing detected".to_string(),
            priority: 4,
        });
    }

    if tank_count >= 2 && !already_built("Lord Dominik's Regards") {
        suggestions.push(SuggestedItem {
            name: "Lord Dominik's Regards".to_string(),
            reason: format!("{} tanks — armor pen", tank_count),
            priority: 4,
        });
    }

    if assassin_count >= 1 && !already_built("Guardian Angel") {
        suggestions.push(SuggestedItem {
            name: "Guardian Angel".to_string(),
            reason: "Assassination threat".to_string(),
            priority: 5,
        });
    }

    if hard_cc_count >= 1 && !already_built("Mercurial Scimitar") {
        suggestions.push(SuggestedItem {
            name: "Mercurial Scimitar".to_string(),
            reason: "Hard CC threat".to_string(),
            priority: 5,
        });
    }

    if winning && !already_built("Runaan's Hurricane") {
        suggestions.push(SuggestedItem {
            name: "Runaan's Hurricane".to_string(),
            reason: "You're winning — AoE shred".to_string(),
            priority: 5,
        });
    }

    suggestions.sort_by_key(|s| s.priority);
    suggestions
}

fn is_ap_champion(name: &str) -> bool {
    const AP_CHAMPS: &[&str] = &[
        "Lux", "Syndra", "Orianna", "Viktor", "Cassiopeia", "Zoe",
        "Veigar", "Brand", "Zyra", "Karma", "Seraphine", "Xerath",
        "Annie", "Fizz", "LeBlanc", "Ahri", "Akali", "Katarina",
        "Diana", "Morgana", "Twisted Fate", "Malzahar",
    ];
    AP_CHAMPS.contains(&name)
}

fn is_healing_item(name: &str) -> bool {
    matches!(name,
        "Immortal Shieldbow" | "Bloodthirster" | "Blade of the Ruined King" |
        "Ravenous Hydra" | "Sterak's Gage"
    )
}

// ── Team fight tip ─────────────────────────────────────────────────────────

pub fn team_fight_tip(enemies: &[&PlayerData], game_time: f32) -> String {
    let has_assassins = enemies.iter().any(|p| ASSASSIN_CHAMPIONS.contains(&p.champion_name.as_str()));
    let tank_count = enemies.iter().filter(|p| TANK_CHAMPIONS.contains(&p.champion_name.as_str())).count();
    let is_poke_comp = enemies.iter().filter(|p| is_poke_champion(&p.champion_name)).count() >= 3;

    if has_assassins {
        "Enemy assassins — stay behind frontline, don't unstealth into 1v1".to_string()
    } else if tank_count >= 3 {
        "Heavy tank comp — spray from max range, let your team engage first".to_string()
    } else if is_poke_comp {
        "Poke comp — clear waves from safety, engage when poke is on cooldown".to_string()
    } else if game_time > 1500.0 {
        "Late game — protect yourself, stealth flanks for backline access".to_string()
    } else {
        "Stay with your team, unstealth at max spray range on priority target".to_string()
    }
}

fn is_poke_champion(name: &str) -> bool {
    const POKE: &[&str] = &["Ezreal", "Jayce", "Nidalee", "Zoe", "Xerath", "Lux", "Karma", "Varus"];
    POKE.contains(&name)
}

// ── Objective tip ──────────────────────────────────────────────────────────

pub fn objective_tip(snapshot: &GameSnapshot) -> Option<String> {
    let events = &snapshot.events.events;
    let game_time = snapshot.game_data.game_time;

    if let Some(dragon) = events.iter().rev().find(|e| e.event_name == "DragonKill") {
        let time_since = game_time - dragon.event_time;
        if time_since < 15.0 {
            let soul_type = dragon.dragon_type.as_deref().unwrap_or("Unknown");
            let priority = match soul_type {
                "Infernal" | "Mountain" => "High priority",
                "Ocean" | "Chemtech" => "High priority — healing/sustain",
                _ => "Take if safe",
            };
            return Some(format!("Dragon killed — {} soul building. {}", soul_type, priority));
        }
    }

    if game_time > 1140.0 && game_time < 1260.0 {
        return Some("Baron spawning soon — group and contest or ward and disengage".to_string());
    }

    if game_time > 480.0 && game_time < 1200.0 {
        if events.iter().any(|e| e.event_name == "HeraldKill") {
            return None;
        }
        return Some("Rift Herald active — ask jungler to secure for tower pressure".to_string());
    }

    None
}

// ── Core build items ───────────────────────────────────────────────────────

pub(crate) fn core_build_items(path: &BuildPath, built: &[String], yun_tal_viable: bool) -> Vec<SuggestedItem> {
    let core: &[(&str, &str)] = match path {
        BuildPath::CritLethality => &[
            ("The Collector",     "Core lethality — execute syncs with poison"),
            ("Fiendhunter Bolts", "Core attack speed + on-hit damage"),
            ("Infinity Edge",     "Crit amplifier — big spike here"),
        ],
        BuildPath::PureCrit => {
            if yun_tal_viable {
                &[
                    ("Yun Tal Wildarrows", "Core crit — viable, 1300g+ first back"),
                    ("Fiendhunter Bolts",  "Core attack speed + on-hit damage"),
                    ("Infinity Edge",      "Crit amplifier — big spike here"),
                ]
            } else {
                &[
                    ("The Collector",     "Core lethality — Yun Tal not viable this game"),
                    ("Fiendhunter Bolts", "Core attack speed + on-hit damage"),
                    ("Infinity Edge",     "Crit amplifier — big spike here"),
                ]
            }
        }
        BuildPath::OnHit => &[
            ("Blade of the Ruined King", "Core on-hit — % HP damage vs tanks"),
            ("Kraken Slayer",            "Core on-hit — true damage every 3rd hit"),
            ("Runaan's Hurricane",       "Core on-hit — multi-target spray"),
        ],
    };

    core.iter()
        .enumerate()
        .filter(|(_, (name, _))| !built.contains(&name.to_string()))
        .map(|(i, (name, reason))| SuggestedItem {
            name: name.to_string(),
            reason: reason.to_string(),
            priority: (i + 1) as u8,
        })
        .collect()
}

// ── Main entry point ───────────────────────────────────────────────────────

pub fn generate_advice(
    snapshot: &GameSnapshot,
    state: &mut EngineState,
    kb: &KnowledgeBase,
) -> GameAdvice {
    if state.summoner_name.is_none() {
        state.summoner_name = Some(snapshot.active_player.summoner_name.clone());
    }
    let my_name = state.summoner_name.as_deref().unwrap_or("");

    let me = snapshot.all_players.iter().find(|p| p.summoner_name == my_name);
    let my_team = me.map(|p| p.team.as_str()).unwrap_or("ORDER");

    let enemies: Vec<&PlayerData> = snapshot.all_players.iter()
        .filter(|p| p.team != my_team)
        .collect();

    let ally_support = snapshot.all_players.iter()
        .find(|p| p.team == my_team && p.position == "SUPPORT" && p.summoner_name != my_name);

    let enemy_bot: Vec<&PlayerData> = snapshot.all_players.iter()
        .filter(|p| p.team != my_team && (p.position == "BOTTOM" || p.position == "SUPPORT"))
        .collect();

    let my_item_ids: Vec<u32> = me.map(|p| p.items.iter().map(|i| i.item_id).collect()).unwrap_or_default();
    let my_built: Vec<String> = me.map(|p| p.items.iter().map(|i| i.display_name.clone()).collect()).unwrap_or_default();
    let my_scores = me.map(|p| p.scores.clone()).unwrap_or(crate::game_state::Scores {
        kills: 0, deaths: 0, assists: 0, creep_score: 0,
    });

    let path = select_build_path(&enemies, state);
    let first_back_note = check_first_back(snapshot, &my_item_ids, state);
    state.previous_item_ids = my_item_ids;

    let mut suggested = core_build_items(&path, &my_built, state.yun_tal_viable);
    suggested.extend(score_situational_items(&enemies, &my_built, &path, &my_scores));

    let support_tip = ally_support.and_then(|s| {
        kb.synergies.iter().find(|e| e.champion == s.champion_name).map(|e| e.tip.clone())
    });

    let lane_tip = {
        let tips: Vec<String> = enemy_bot.iter()
            .filter_map(|p| {
                kb.matchups.iter().find(|m| m.name == p.champion_name)
                    .map(|m| format!("vs {}: {}", p.champion_name, m.tip))
            })
            .collect();
        if tips.is_empty() { None } else { Some(tips.join(" | ")) }
    };

    GameAdvice {
        build_path: path,
        built_items: my_built,
        suggested_items: suggested,
        first_back_note,
        support_tip,
        lane_tip,
        team_fight_tip: team_fight_tip(&enemies, snapshot.game_data.game_time),
        objective_tip: objective_tip(snapshot),
        game_time: snapshot.game_data.game_time,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::{PlayerData, Scores};

    fn make_player(champion: &str, team: &str, position: &str) -> PlayerData {
        PlayerData {
            champion_name: champion.to_string(),
            summoner_name: format!("{}_player", champion),
            team: team.to_string(),
            items: vec![],
            level: 8,
            position: position.to_string(),
            scores: Scores { kills: 1, deaths: 1, assists: 0, creep_score: 80 },
            is_dead: false,
        }
    }

    #[test]
    fn test_build_path_defaults_to_crit_lethality_for_mixed_comp() {
        let enemies = vec![
            make_player("Caitlyn", "CHAOS", "BOTTOM"),
            make_player("Lux", "CHAOS", "SUPPORT"),
            make_player("Zed", "CHAOS", "MIDDLE"),
        ];
        let refs: Vec<&PlayerData> = enemies.iter().collect();
        let mut state = EngineState::default();
        assert_eq!(select_build_path(&refs, &mut state), BuildPath::CritLethality);
    }

    #[test]
    fn test_build_path_on_hit_for_3_tanks() {
        let enemies = vec![
            make_player("Malphite", "CHAOS", "TOP"),
            make_player("Nautilus", "CHAOS", "SUPPORT"),
            make_player("Ornn", "CHAOS", "JUNGLE"),
            make_player("Jinx", "CHAOS", "BOTTOM"),
        ];
        let refs: Vec<&PlayerData> = enemies.iter().collect();
        let mut state = EngineState::default();
        assert_eq!(select_build_path(&refs, &mut state), BuildPath::OnHit);
    }

    #[test]
    fn test_build_path_locked_after_first_selection() {
        let enemies = vec![make_player("Caitlyn", "CHAOS", "BOTTOM")];
        let refs: Vec<&PlayerData> = enemies.iter().collect();
        let mut state = EngineState::default();
        select_build_path(&refs, &mut state);
        let tanks = vec![
            make_player("Malphite", "CHAOS", "TOP"),
            make_player("Nautilus", "CHAOS", "SUPPORT"),
            make_player("Ornn", "CHAOS", "JUNGLE"),
        ];
        let tank_refs: Vec<&PlayerData> = tanks.iter().collect();
        assert_eq!(select_build_path(&tank_refs, &mut state), BuildPath::CritLethality);
    }

    #[test]
    fn test_yun_tal_blocked_when_low_gold_first_back() {
        use crate::game_state::{GameSnapshot, ActivePlayer, ChampionStats, Events, GameData};
        let mut state = EngineState::default();
        state.peak_gold_before_first_buy = 950.0;
        state.previous_item_ids = vec![1055];
        let snapshot = GameSnapshot {
            active_player: ActivePlayer {
                current_gold: 200.0,
                summoner_name: "TestPlayer".to_string(),
                level: 8,
                champion_stats: ChampionStats { current_health: 900.0, max_health: 1400.0 },
            },
            all_players: vec![],
            events: Events { events: vec![] },
            game_data: GameData { game_time: 300.0, game_mode: "CLASSIC".to_string() },
        };
        let note = check_first_back(&snapshot, &[1055, 1036], &mut state);
        assert!(note.is_some());
        assert!(!state.yun_tal_viable);
        assert!(note.unwrap().contains("Yun Tal not viable"));
    }

    #[test]
    fn test_yun_tal_viable_when_1300_gold_first_back() {
        use crate::game_state::{GameSnapshot, ActivePlayer, ChampionStats, Events, GameData};
        let mut state = EngineState::default();
        state.peak_gold_before_first_buy = 1450.0;
        state.previous_item_ids = vec![1055];
        let snapshot = GameSnapshot {
            active_player: ActivePlayer {
                current_gold: 150.0,
                summoner_name: "TestPlayer".to_string(),
                level: 9,
                champion_stats: ChampionStats { current_health: 800.0, max_health: 1400.0 },
            },
            all_players: vec![],
            events: Events { events: vec![] },
            game_data: GameData { game_time: 350.0, game_mode: "CLASSIC".to_string() },
        };
        let note = check_first_back(&snapshot, &[1055, 1038], &mut state);
        assert!(note.is_some());
        assert!(state.yun_tal_viable);
        assert!(note.unwrap().contains("Yun Tal or Collector available"));
    }

    #[test]
    fn test_situational_mortal_reminder_for_healer() {
        let enemies = vec![
            make_player("Soraka", "CHAOS", "SUPPORT"),
            make_player("Caitlyn", "CHAOS", "BOTTOM"),
        ];
        let refs: Vec<&PlayerData> = enemies.iter().collect();
        let scores = Scores { kills: 2, deaths: 1, assists: 1, creep_score: 80 };
        let suggestions = score_situational_items(&refs, &[], &BuildPath::CritLethality, &scores);
        assert!(suggestions.iter().any(|s| s.name == "Mortal Reminder"));
    }

    #[test]
    fn test_situational_ldr_for_2_tanks() {
        let enemies = vec![
            make_player("Malphite", "CHAOS", "TOP"),
            make_player("Nautilus", "CHAOS", "SUPPORT"),
            make_player("Jinx", "CHAOS", "BOTTOM"),
        ];
        let refs: Vec<&PlayerData> = enemies.iter().collect();
        let scores = Scores { kills: 1, deaths: 1, assists: 0, creep_score: 80 };
        let suggestions = score_situational_items(&refs, &[], &BuildPath::CritLethality, &scores);
        assert!(suggestions.iter().any(|s| s.name == "Lord Dominik's Regards"));
    }

    #[test]
    fn test_situational_maw_for_3_ap_on_crit_path() {
        let enemies = vec![
            make_player("Lux", "CHAOS", "SUPPORT"),
            make_player("Viktor", "CHAOS", "MIDDLE"),
            make_player("Syndra", "CHAOS", "TOP"),
        ];
        let refs: Vec<&PlayerData> = enemies.iter().collect();
        let scores = Scores { kills: 1, deaths: 1, assists: 0, creep_score: 80 };
        let suggestions = score_situational_items(&refs, &[], &BuildPath::CritLethality, &scores);
        assert!(suggestions.iter().any(|s| s.name == "Maw of Malmortius"));
    }

    #[test]
    fn test_situational_wits_end_for_3_ap_on_hit_path() {
        let enemies = vec![
            make_player("Lux", "CHAOS", "SUPPORT"),
            make_player("Viktor", "CHAOS", "MIDDLE"),
            make_player("Syndra", "CHAOS", "TOP"),
        ];
        let refs: Vec<&PlayerData> = enemies.iter().collect();
        let scores = Scores { kills: 1, deaths: 1, assists: 0, creep_score: 80 };
        let suggestions = score_situational_items(&refs, &[], &BuildPath::OnHit, &scores);
        assert!(suggestions.iter().any(|s| s.name == "Wit's End"));
    }

    #[test]
    fn test_already_built_items_not_suggested() {
        let enemies = vec![make_player("Soraka", "CHAOS", "SUPPORT")];
        let refs: Vec<&PlayerData> = enemies.iter().collect();
        let scores = Scores { kills: 1, deaths: 1, assists: 0, creep_score: 80 };
        let built = vec!["Mortal Reminder".to_string()];
        let suggestions = score_situational_items(&refs, &built, &BuildPath::CritLethality, &scores);
        assert!(!suggestions.iter().any(|s| s.name == "Mortal Reminder"));
    }

    #[test]
    fn test_core_build_excludes_already_built_items() {
        let built = vec!["The Collector".to_string()];
        let items = core_build_items(&BuildPath::CritLethality, &built, true);
        assert!(!items.iter().any(|i| i.name == "The Collector"));
        assert!(items.iter().any(|i| i.name == "Fiendhunter Bolts"));
    }

    #[test]
    fn test_knowledge_base_loads_without_panic() {
        let kb = load_knowledge_base();
        assert!(!kb.items.is_empty());
        assert!(!kb.synergies.is_empty());
        assert!(!kb.matchups.is_empty());
    }
}
