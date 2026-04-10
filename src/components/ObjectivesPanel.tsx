import { GameAdvice } from '../types';

interface Props {
  advice: GameAdvice;
}

export function ObjectivesPanel({ advice }: Props) {
  const hasContent = advice.team_fight_tip || advice.objective_tip;
  if (!hasContent) return null;

  return (
    <div className="panel">
      <div className="panel-header">
        <span className="panel-title">TEAM / OBJECTIVES</span>
      </div>

      {advice.team_fight_tip && (
        <div className="tip-row">
          <span className="tip-label">Fight</span>
          <span className="tip-text">{advice.team_fight_tip}</span>
        </div>
      )}

      {advice.objective_tip && (
        <div className="tip-row objective">
          <span className="tip-label">Objective</span>
          <span className="tip-text">{advice.objective_tip}</span>
        </div>
      )}
    </div>
  );
}
