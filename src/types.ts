export type BuildPath = 'CritLethality' | 'PureCrit' | 'OnHit';

export interface SuggestedItem {
  name: string;
  reason: string;
  priority: number;
}

export interface GameAdvice {
  build_path: BuildPath;
  built_items: string[];
  suggested_items: SuggestedItem[];
  first_back_note: string | null;
  support_tip: string | null;
  lane_tip: string | null;
  team_fight_tip: string;
  objective_tip: string | null;
  game_time: number;
}
