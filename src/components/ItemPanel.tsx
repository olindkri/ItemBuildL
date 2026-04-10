import { GameAdvice } from '../types';

interface Props {
  advice: GameAdvice;
}

const BUILD_PATH_LABEL: Record<string, string> = {
  CritLethality: 'Crit / Lethality',
  PureCrit: 'Pure Crit',
  OnHit: 'On-Hit',
};

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}

export function ItemPanel({ advice }: Props) {
  const coreItems = advice.suggested_items.filter(i => i.priority <= 3);
  const situationalItems = advice.suggested_items.filter(i => i.priority > 3);

  return (
    <div className="panel">
      <div className="panel-header">
        <span className="panel-title">ITEMS</span>
        <span className="badge">{BUILD_PATH_LABEL[advice.build_path]} · {formatTime(advice.game_time)}</span>
      </div>

      {advice.first_back_note && (
        <div className="first-back-note">
          {advice.first_back_note}
        </div>
      )}

      {advice.built_items.length > 0 && (
        <div className="item-row">
          <span className="item-label">Built</span>
          <span className="item-list">{advice.built_items.join(' · ')}</span>
        </div>
      )}

      {coreItems.map((item, i) => (
        <div key={item.name} className="item-row suggested">
          <span className="item-priority">{i === 0 ? 'Next' : i === 1 ? 'Then' : `${i + 1}th`}</span>
          <span className="item-name">{item.name}</span>
          <span className="item-reason">{item.reason}</span>
        </div>
      ))}

      {situationalItems.length > 0 && (
        <>
          <div className="section-divider">Situational</div>
          {situationalItems.map(item => (
            <div key={item.name} className="item-row situational">
              <span className="item-name">{item.name}</span>
              <span className="item-reason">{item.reason}</span>
            </div>
          ))}
        </>
      )}
    </div>
  );
}
