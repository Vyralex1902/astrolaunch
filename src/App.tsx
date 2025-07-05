import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

type AppInfo = {
  name: string;
  path: string;
};

type BuiltInCommand = {
  name: string;
  action: () => void;
  isCalc?: boolean;
  calcResult?: string;
};

type SearchResult = AppInfo | BuiltInCommand;

function isAppInfo(item: SearchResult): item is AppInfo {
  return (item as AppInfo).path !== undefined;
}

const emojiMap: Record<string, string> = {
  smile: '😊',
  laugh: '😂',
  thumbs_up: '👍',
  heart: '❤️',
  fire: '🔥',
  star: '⭐',
  party: '🎉',
  rocket: '🚀',
  coffee: '☕',
  sun: '☀️',
  moon: '🌙',
  wink: '😉',
  cry: '😢',
  angry: '😠',
  kiss: '😘',
  blush: '😊',
  thinking: '🤔',
  clap: '👏',
  ok_hand: '👌',
  eyes: '👀',
  raised_hands: '🙌',
  muscle: '💪',
  poop: '💩',
  ghost: '👻',
  broken_heart: '💔',
  dizzy: '😵',
  heart_eyes: '😍',
  sleeping: '😴',
  sunglasses: '😎',
  confetti_ball: '🎊',
  balloon: '🎈',
  cake: '🍰',
  beer: '🍺',
  pizza: '🍕',
  soccer: '⚽',
  basketball: '🏀',
  guitar: '🎸',
  microphone: '🎤',
  camera: '📷',
  phone: '📱',
  laptop: '💻',
  book: '📚',
  moneybag: '💰',
  warning: '⚠️',
  check_mark: '✅',
  cross_mark: '❌',
  thumbs_down: '👎',
};

export default function App() {
  const [apps, setApps] = useState<AppInfo[]>([]);
  const [query, setQuery] = useState('');
  const [filtered, setFiltered] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [emojiMode, setEmojiMode] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const [calcResult, setCalcResult] = useState<string | null>(null);

  useEffect(() => {
    inputRef.current?.focus();
    invoke<AppInfo[]>('list_apps')
      .then(setApps)
      .catch((err) => console.error('Failed to fetch apps:', err));
  }, []);

  useEffect(() => {
    if (query.trim() === '') {
      setCalcResult(null);
      return;
    }
    if (/^[\d\s+\-*/().]+$/.test(query.trim())) {
      invoke<number>('calculate_expression', { expression: query.trim() })
        .then((result) => {
          setCalcResult(result.toString());
        })
        .catch(() => {
          setCalcResult(null);
        });
    } else {
      setCalcResult(null);
    }
  }, [query]);

  // Get emojis filtered by query in emojiMode
  function getFilteredEmojis(query: string): BuiltInCommand[] {
    const q = query.toLowerCase();
    return Object.entries(emojiMap)
      .filter(([name]) => name.includes(q))
      .map(([name, emoji]) => ({
        name: `${emoji} ${name}`,
        action: () => {
          navigator.clipboard.writeText(emoji).catch(console.error);
          setEmojiMode(false);
          setQuery('');
        },
      }));
  }

  // Multimedia commands parser and built-in
  function getMultimediaCommands(query: string): BuiltInCommand[] {
    const q = query.toLowerCase();

    // Regex to extract number before %
    const percentMatch = q.match(/(\d{1,3})\s*%/);
    const number = percentMatch ? Math.min(Math.max(parseInt(percentMatch[1]), 0), 100) : null;

    const cmds: BuiltInCommand[] = [];

    if (!q) return cmds;

    if (q.startsWith("set volume") && number !== null) {
      cmds.push({
        name: `Set volume to ${number}%`,
        action: () => invoke('set_volume', { volume: number }),
      });
    } else if (q.startsWith("increase volume") && number !== null) {
      cmds.push({
        name: `Increase volume by ${number}%`,
        action: () => invoke('increase_volume', { delta: number }),
      });
    } else if (q.startsWith("decrease volume") && number !== null) {
      cmds.push({
        name: `Decrease volume by ${number}%`,
        action: () => invoke('decrease_volume', { delta: number }),
      });
    }

    if (q.startsWith("set brightness") && number !== null) {
      cmds.push({
        name: `Set brightness to ${number}%`,
        action: () => invoke('set_brightness', { brightness: number }),
      });
    } else if (q.startsWith("increase brightness") && number !== null) {
      cmds.push({
        name: `Increase brightness by ${number}%`,
        action: () => invoke('increase_brightness', { delta: number }),
      });
    } else if (q.startsWith("decrease brightness") && number !== null) {
      cmds.push({
        name: `Decrease brightness by ${number}%`,
        action: () => invoke('decrease_brightness', { delta: number }),
      });
    }

    if (q.startsWith("play")) {
      cmds.push({ name: "Play media", action: () => invoke('media_play') });
    }
    if (q.startsWith("pause")) {
      cmds.push({ name: "Pause media", action: () => invoke('media_pause') });
    }
    if (q.startsWith("skip")) {
      cmds.push({ name: "Skip track", action: () => invoke('media_skip') });
    }
    if (q.startsWith("previous")) {
      cmds.push({ name: "Previous track", action: () => invoke('media_previous') });
    }

    return cmds;
  }

  // Get built-in commands depending on emojiMode
  function getBuiltInCommands(query: string): BuiltInCommand[] {
    const q = query.trim();

    if (emojiMode) {
      // Only emojis when in emoji mode
      return getFilteredEmojis(q);
    }

    // Not in emoji mode, show "emoji" command to enter emoji mode
    const builtIn: BuiltInCommand[] = [
      {
        name: 'emoji',
        action: () => {
          setEmojiMode(true);
          setQuery('');
          setSelectedIndex(-1);
        },
      },
    ];

    // Insert multimedia commands only if query includes '%' or contains trigger words
    if (!emojiMode) {
      const multimediaCommands = getMultimediaCommands(q);
      const triggers = ['set', 'increase', 'decrease', 'play', 'pause', 'skip', 'previous'];
      if (q.includes('%') || triggers.some(t => q.includes(t))) {
        builtIn.push(...multimediaCommands);
      }
    }

    if (['minimize', 'min'].includes(q.toLowerCase())) {
      builtIn.push({ name: 'Minimize Window', action: () => invoke('minimize_window') });
    }
    if (['maximize', 'max'].includes(q.toLowerCase())) {
      builtIn.push({ name: 'Maximize Window', action: () => invoke('maximize_window') });
    }
    if (['resize', 'resize 80'].includes(q.toLowerCase())) {
      builtIn.push({ name: 'Resize to 80%', action: () => invoke('resize_window_80') });
    }
    if (['close', 'quit'].includes(q.toLowerCase())) {
      builtIn.push({ name: 'Close Window', action: () => invoke('close_window') });
    }
    if (calcResult !== null && q !== calcResult) {
      builtIn.push({
        name: `Calculate: ${q} = ${calcResult}`,
        action: () => {
          setQuery(calcResult);
          setCalcResult(null);
        },
        isCalc: true,
        calcResult,
      });
    }

    if (q.toLowerCase().startsWith('yt:')) {
      const term = q.substring(3).trim();
      if (term.length > 0) {
        builtIn.push({
          name: `Search YouTube for "${term}"`,
          action: () =>
            invoke('search_web', { query: `https://www.youtube.com/results?search_query=${encodeURIComponent(term)}` }).catch(console.error),
        });
      }
    } else if (q.length > 0) {
      builtIn.push({
        name: `Search the web for "${q}"`,
        action: () => invoke('search_web', { query: q }).catch(console.error),
      });
    }

    return builtIn;
  }

  useEffect(() => {
    const trimmedQuery = query.trim();
    const builtInMatches = getBuiltInCommands(trimmedQuery);
    const appMatches = emojiMode ? [] : apps.filter((app) =>
      app.name.toLowerCase().includes(trimmedQuery.toLowerCase())
    );
    const newFiltered = [...builtInMatches, ...appMatches];
    setFiltered(newFiltered);
    setSelectedIndex(newFiltered.length > 0 ? 0 : -1);
  }, [query, apps, calcResult, emojiMode]);

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Escape') {
      if (!emojiMode) {
        invoke('close_window_command').catch(console.error);
        e.preventDefault();
        return;
      }
    }

    if (filtered.length === 0) {
      return;
    }

    if (emojiMode) {
      if (e.key === 'Backspace' && query === '') {
        // Exit emoji mode on backspace with empty input
        setEmojiMode(false);
        setSelectedIndex(-1);
        e.preventDefault();
        return;
      }
      if (e.key === 'Enter') {
        if (selectedIndex === -1) return;
        const selected = filtered[selectedIndex];
        if (!selected) return;
        // In emoji mode, pressing enter copies emoji to clipboard and exits emoji mode
        if ('action' in selected) {
          selected.action();
          e.preventDefault();
        }
        return;
      }
    }

    if (e.key === 'ArrowDown') {
      setSelectedIndex((i) => (i + 1) % filtered.length);
      e.preventDefault();
    } else if (e.key === 'ArrowUp') {
      setSelectedIndex((i) => (i - 1 + filtered.length) % filtered.length);
      e.preventDefault();
    } else if (e.key === 'Enter') {
      if (selectedIndex === -1) return;
      const selected = filtered[selectedIndex];
      if (!selected) return;

      if (emojiMode) {
        // If selected is "emoji" command, enter emoji mode and clear input
        if (selected.name === 'emoji') {
          setEmojiMode(true);
          setQuery('');
          setSelectedIndex(-1);
          e.preventDefault();
          return;
        }
      }

      if (isAppInfo(selected)) {
        invoke('launch_app', { appName: selected.path }).catch(console.error);
      } else {
        selected.action();
      }
      e.preventDefault();
    }
  }

  return (
    <div
      style={{
        backgroundColor: 'rgba(30, 30, 30, 0.85)',
        color: 'white',
        fontFamily: 'system-ui, sans-serif',
        borderRadius: 12,
        padding: 20,
        width: 600,
        maxHeight: 400,
        overflow: 'hidden',
        boxShadow: '0 10px 30px rgba(0,0,0,0.7)',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <input
        ref={inputRef}
        type="text"
        placeholder={emojiMode ? 'Type emoji name...' : 'Type a command or search...'}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        onKeyDown={onKeyDown}
        style={{
          backgroundColor: 'transparent',
          border: 'none',
          outline: 'none',
          color: 'white',
          fontSize: 20,
          padding: '8px 12px',
          borderRadius: 8,
          marginBottom: 15,
          boxShadow: 'inset 0 0 5px rgba(255, 255, 255, 0.1)',
        }}
      />

      {query.length > 0 && filtered.length > 0 && (
        <ul
          style={{
            listStyle: 'none',
            padding: 0,
            margin: 0,
            overflowY: 'auto',
            flexGrow: 1,
          }}
        >
          {filtered.map((item, i) => (
            <li
              key={isAppInfo(item) ? item.path : item.name}
              onClick={() => {
                if (isAppInfo(item)) {
                  invoke('launch_app', { appName: item.path }).catch(console.error);
                } else {
                  item.action();
                }
              }}
              onMouseEnter={() => setSelectedIndex(i)}
              style={{
                padding: '10px 15px',
                backgroundColor: i === selectedIndex ? '#3a3a3a' : 'transparent',
                borderRadius: 8,
                cursor: 'pointer',
                userSelect: 'none',
              }}
            >
              {item.name}
            </li>
          ))}
        </ul>
      )}

      {query.length > 0 && filtered.length === 0 && (
        <div style={{ color: 'gray', fontStyle: 'italic', padding: 10 }}>
          No apps found.
        </div>
      )}
    </div>
  );
}