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

export default function App() {
  const [apps, setApps] = useState<AppInfo[]>([]);
  const [query, setQuery] = useState('');
  const [filtered, setFiltered] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
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

  function getBuiltInCommands(query: string): BuiltInCommand[] {
    const q = query.trim();

    const builtIn: BuiltInCommand[] = [];

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
    const builtInMatches = getBuiltInCommands(query);
    const appMatches = apps.filter((app) =>
      app.name.toLowerCase().includes(query.toLowerCase())
    );
    setFiltered([...builtInMatches, ...appMatches]);
    setSelectedIndex(0);
  }, [query, apps, calcResult]);

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      setSelectedIndex((i) => (i + 1) % filtered.length);
      e.preventDefault();
    } else if (e.key === 'ArrowUp') {
      setSelectedIndex((i) => (i - 1 + filtered.length) % filtered.length);
      e.preventDefault();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const selected = filtered[selectedIndex];
      if (selected) {
        if (isAppInfo(selected)) {
          invoke('launch_app', { appName: selected.path }).catch(console.error);
        } else {
          selected.action();
        }
      }
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
        placeholder="Type a command or search..."
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