import { GameAdvice } from '../types';

interface Props {
  advice: GameAdvice;
}

export function LanePanel({ advice }: Props) {
  if (!advice.support_tip && !advice.lane_tip) return null;

  return (
    <div className="panel">
      <div className="panel-header">
        <span className="panel-title">LANE</span>
      </div>

      {advice.support_tip && (
        <div className="tip-row">
          <span className="tip-label">Support</span>
          <span className="tip-text">{advice.support_tip}</span>
        </div>
      )}

      {advice.lane_tip && advice.lane_tip.map((tip, i) => (
        <div key={i} className="tip-row">
          <span className="tip-label">Matchup</span>
          <span className="tip-text">{tip}</span>
        </div>
      ))}
    </div>
  );
}
