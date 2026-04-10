import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { GameAdvice } from './types';
import { ItemPanel } from './components/ItemPanel';
import { LanePanel } from './components/LanePanel';
import { ObjectivesPanel } from './components/ObjectivesPanel';

export default function App() {
  const [advice, setAdvice] = useState<GameAdvice | null>(null);
  const [alwaysOnTop, setAlwaysOnTop] = useState(false);

  useEffect(() => {
    const unlistenAdvice = listen<GameAdvice>('game-advice', (event) => {
      setAdvice(event.payload);
    });

    const unlistenIdle = listen('game-idle', () => {
      setAdvice(null);
    });

    return () => {
      unlistenAdvice.then(fn => fn());
      unlistenIdle.then(fn => fn());
    };
  }, []);

  async function toggleAlwaysOnTop() {
    const next = !alwaysOnTop;
    await invoke('toggle_always_on_top', { enable: next });
    setAlwaysOnTop(next);
  }

  return (
    <div className="app">
      <div className="titlebar">
        <span className="app-title">TWITCH ADVISOR</span>
        <div className="titlebar-right">
          {advice ? (
            <span className="status-active">● In Game</span>
          ) : (
            <span className="status-idle">○ Waiting for game...</span>
          )}
          <button
            className={`pin-btn ${alwaysOnTop ? 'pinned' : ''}`}
            onClick={toggleAlwaysOnTop}
            title="Always on top"
          >
            📌
          </button>
        </div>
      </div>

      <div className="content">
        {advice ? (
          <>
            <ItemPanel advice={advice} />
            <LanePanel advice={advice} />
            <ObjectivesPanel advice={advice} />
          </>
        ) : (
          <div className="idle-message">
            Start a League of Legends game to begin receiving advice.
          </div>
        )}
      </div>
    </div>
  );
}
