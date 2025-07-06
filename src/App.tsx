import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

type AppInfo = {
  name: string;
  path: string;
};

import webIcon from './assets/white-web.png';
import iVolIcon from './assets/white-volup.png';
import dVolIcon from './assets/white-voldown.png';
import mVolIcon from './assets/white-mute.png';
import emojiIcon from './assets/white-emoji.png';
import youIcon from './assets/youtube.png';
import playIcon from './assets/white-play.png';
import nextIcon from './assets/white-next.png';
import backIcon from './assets/white-back.png';
import pauseIcon from './assets/white-pause.png';
import brightIcon from './assets/white-bright.png';
import calcIcon from './assets/white-calc.png';
import clipIcon from './assets/white-clip.png';

type BuiltInCommand = {
  name: string;
  action: () => void;
  isCalc?: boolean;
  calcResult?: string;
};

type FileSearchItem = {
  name: string;
  path: string;
  action: () => void;
};
type SearchResult = AppInfo | BuiltInCommand | FileSearchItem;

function isAppInfo(item: SearchResult): item is AppInfo {
  // FileSearchItem and AppInfo both have path, but AppInfo does not have action
  return (item as AppInfo).path !== undefined && !(item as FileSearchItem).action;
}
function isFileSearchItem(item: SearchResult): item is FileSearchItem {
  return (item as FileSearchItem).action !== undefined && (item as FileSearchItem).path !== undefined;
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
  const [fileSearchResults, setFileSearchResults] = useState<FileSearchItem[]>([]);

  const [clipboardMode, setClipboardMode] = useState(false);
  const [clipboardItems, setClipboardItems] = useState<string[]>([]);
  // Snippet state
  const [snippets, setSnippets] = useState<{ name: string; content: string }[]>([]);
  const [snippetMode, setSnippetMode] = useState(false);
  // Fetch clipboard history from backend
  async function fetchClipboardHistory() {
    try {
      const items = await invoke<string[]>('get_clipboard_history');
      setClipboardItems(items);
    } catch (e) {
      console.error('Failed to fetch clipboard history:', e);
      setClipboardItems([]);
    }
  }



  useEffect(() => {
    inputRef.current?.focus();
    invoke<AppInfo[]>('list_apps')
      .then(setApps)
      .catch((err) => console.error('Failed to fetch apps:', err));
    // Fetch snippets
    invoke<[string, string][]>('get_snippets')
      .then((data) => {
        const formatted = data.map(([name, content]) => ({ name, content }));
        setSnippets(formatted);
      })
      .catch((err) => console.error('Failed to fetch snippets:', err));
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

  useEffect(() => {
    if (!query.toLowerCase().startsWith('search file:')) {
      setFileSearchResults([]);
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
    }
    else if (q.startsWith("mute")) {
      cmds.push({
        name: `Mute volume`,
        action: () => invoke('mute_volume'),
      });
    }
    else if (q.startsWith("increase volume") && number !== null) {
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

  // Get built-in commands depending on emojiMode and clipboardMode
  function getBuiltInCommands(query: string): BuiltInCommand[] {
    const q = query.trim().toLowerCase();

    if (clipboardMode) return [];
    if (snippetMode) return [];

    if (emojiMode) {
      return getFilteredEmojis(q);
    }

    // Base static built-in commands
    const allCommands: BuiltInCommand[] = [
      {
        name: 'Clipboard history',
        action: () => {
          if (clipboardItems.length === 0) return;
          setClipboardMode(true);
          setQuery('');
          fetchClipboardHistory();
          setSelectedIndex(0);
        },
      },
      {
        name: 'Snippets',
        action: () => {
          if (snippets.length === 0) return;
          setSnippetMode(true);
          setQuery('');
          setSelectedIndex(0);
        },
      },
      {
        name: 'Emoji',
        action: () => {
          setEmojiMode(true);
          setQuery('');
          setSelectedIndex(-1);
        },
      },
      // Add multimedia commands, window commands, and others below:
    ];

    // Add multimedia commands if query contains trigger words
    const triggers = ['set', 'increase', 'decrease', 'play', 'pause', 'skip', 'previous', 'mute'];
    if (triggers.some(t => q.includes(t))) {
      allCommands.push(...getMultimediaCommands(query));
    }

    // Window commands
    if (q.includes('minimize') || q.includes('min')) {
      allCommands.push({ name: 'Minimize Window', action: () => invoke('minimize_window') });
    }
    if (q.includes('maximize') || q.includes('max')) {
      allCommands.push({ name: 'Maximize Window', action: () => invoke('maximize_window') });
    }
    if (q.includes('resize') || q.includes('resize 80')) {
      allCommands.push({ name: 'Resize to 80%', action: () => invoke('resize_window_80') });
    }
    if (q.includes('close') || q.includes('quit')) {
      allCommands.push({ name: 'Close Window', action: () => invoke('close_window') });
    }

    // Calculator result command
    if (calcResult !== null && q !== calcResult.toLowerCase()) {
      allCommands.push({
        name: `Calculate: ${query} = ${calcResult}`,
        action: () => {
          setQuery(calcResult);
          setCalcResult(null);
        },
        isCalc: true,
        calcResult,
      });
    }

    // YouTube search
    if (q.startsWith('yt:')) {
      const term = query.substring(3).trim();
      if (term.length > 0) {
        allCommands.push({
          name: `Search YouTube for "${term}"`,
          action: () => invoke('search_web', { query: `https://www.youtube.com/results?search_query=${encodeURIComponent(term)}` }).catch(console.error),
        });
      }
    } else if (query.length > 0 && !q.includes(':')) {
      allCommands.push({
        name: `Search the web for "${query}"`,
        action: () => invoke('search_web', { query }).catch(console.error),
      });
    }

    // Open link directly if query starts with http or https
    if (q.startsWith('http://') || q.startsWith('https://')) {
      allCommands.push({
        name: `Open link "${query}"`,
        action: () => invoke('open_link', { url: query }).catch(console.error),
      });
    }

    // Snippet insert command
    snippets.forEach(({ name, content }) => {
      if (name.toLowerCase().includes(q)) {
        allCommands.push({
          name: `Insert snippet "${name}"`,
          action: () => {
            navigator.clipboard.writeText(content).catch(console.error);
            setQuery('');
          },
        });
      }
    });

    // File search command
    if (q.includes('search file')) {
      const term = query.toLowerCase().split('search file')[1]?.trim();
      if (term && term.length > 0) {
        allCommands.push({
          name: `Search local files for "${term}"`,
          action: () => {
            invoke<string[]>('search_files', { query: term })
              .then((results) => {
                const items = results.map(path => ({
                  name: path,
                  path,
                  action: () => invoke('launch_app', { appName: path }).catch(console.error),
                }));
                setFileSearchResults(items);
              })
              .catch(() => {
                alert('File search failed.');
                setFileSearchResults([]);
              });
          }
        });
      }
    }

    // Filter commands by substring match, score by index of query in name (lower index better)
    const matchedCommands = allCommands
      .map(cmd => {
        const idx = cmd.name.toLowerCase().indexOf(q);
        return { cmd, idx };
      })
      .filter(({ idx }) => idx !== -1)
      .sort((a, b) => a.idx - b.idx)
      .map(({ cmd }) => cmd);

    return matchedCommands;
  }

  useEffect(() => {
    if (clipboardMode) {
      // Filter clipboard items by full query string
      const filteredClipboardCommands: BuiltInCommand[] = clipboardItems
        .filter(item => item.toLowerCase().includes(query.toLowerCase()))
        .map(text => ({
          name: text.length > 60 ? text.slice(0, 60) + '...' : text,
          action: () => {
            navigator.clipboard.writeText(text).catch(console.error);
            setClipboardMode(false);
            setQuery('');
          },
        }));

      setFiltered(filteredClipboardCommands);
      setSelectedIndex(filteredClipboardCommands.length > 0 ? 0 : -1);
    }
    else if (snippetMode) {
      const filteredSnippetCommands: BuiltInCommand[] = snippets
        .filter(snippet => snippet.name.toLowerCase().includes(query.toLowerCase()))
        .map(snippet => ({
          name: `Insert snippet "${snippet.name}"`,
          action: () => {
            navigator.clipboard.writeText(snippet.content).catch(console.error);
            setSnippetMode(false);
            setQuery('');
          },
        }));

      setFiltered(filteredSnippetCommands);
      setSelectedIndex(filteredSnippetCommands.length > 0 ? 0 : -1);
    }
    else {
      const trimmedQuery = query.trim();
      const builtInMatches = getBuiltInCommands(trimmedQuery);
      const appMatches = emojiMode ? [] : apps.filter(app =>
        app.name.toLowerCase().includes(trimmedQuery.toLowerCase())
      );
      const combined = [...builtInMatches, ...appMatches, ...fileSearchResults];
      const newFiltered = combined.slice(0, 8);
      setFiltered(newFiltered);
      setSelectedIndex(newFiltered.length > 0 ? 0 : -1);
    }
  }, [query, apps, calcResult, emojiMode, fileSearchResults, clipboardMode, clipboardItems, snippets, snippetMode]);

  function onKeyDown(e: React.KeyboardEvent) {
    if (clipboardMode && e.key === 'Backspace' && query.trim() === '') {
      setClipboardMode(false);
      setClipboardItems([]);
      e.preventDefault();
      return;
    }
    if (snippetMode && e.key === 'Backspace' && query.trim() === '') {
      setSnippetMode(false);
      e.preventDefault();
      return;
    }

    console.log('Key down:', e.key);

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
        console.log('Enter key pressed in emojiMode');
        console.log('selectedIndex:', selectedIndex);
        console.log('filtered:', filtered);
        if (selectedIndex === -1) return;
        const selected = filtered[selectedIndex];
        console.log('Selected item:', selected);
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

      // Prevent processing if "Snippets" or "Clipboard history" are selected but have no data
      if (
        (selected.name === 'Snippets' && snippets.length === 0) ||
        (selected.name === 'Clipboard history' && clipboardItems.length === 0)
      ) {
        e.preventDefault();
        return;
      }

      // General handler for items with an action function
      if ('action' in selected && typeof selected.action === 'function') {
        selected.action();
        e.preventDefault();
        return;
      }

      // (Old built-in handling removed)
    }
  }

  function getIconForItem(name: string): string | null {
    const lowerName = name.toLowerCase();
    if (lowerName.includes('search the web')) {
      return webIcon;
    }
    else if (lowerName.includes('https://') || lowerName.includes('http://')) {
      return webIcon;
    } else if (lowerName.includes('set volume')) {
      return iVolIcon;
    } else if (lowerName.includes('increase volume')) {
      return iVolIcon;
    }
    else if (lowerName.includes('decrease volume')) {
      return dVolIcon;
    }
    else if (lowerName.includes('mute volume')) {
      return mVolIcon;
    }
    else if (lowerName.includes('emoji')) {
      return emojiIcon;
    }
    else if (lowerName.includes('search youtube')) {
      return youIcon;
    }
    else if (lowerName.includes('play media')) {
      return playIcon;
    } else if (lowerName.includes('pause media')) {
      return pauseIcon;
    } else if (lowerName.includes('skip track')) {
      return nextIcon;
    } else if (lowerName.includes('previous track')) {
      return backIcon;
    } else if (lowerName.includes('set brightness')) {
      return brightIcon;
    } else if (lowerName.includes('increase brightness')) {
      return brightIcon;
    } else if (lowerName.includes('decrease brightness')) {
      return brightIcon;
    } else if (lowerName.includes('calculate')) {
      return calcIcon;
    } else if (lowerName.includes('clipboard')) {
      return clipIcon;
    }
    return null;
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
        placeholder={clipboardMode ? 'Search clipboard history...' : emojiMode ? 'Type emoji name...' : 'Type a command or search...'}
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
              key={
                isAppInfo(item)
                  ? item.path
                  : isFileSearchItem(item)
                    ? item.path
                    : item.name
              }
              onClick={() => {
                if (isAppInfo(item)) {
                  invoke('launch_app', { appName: item.path }).catch(console.error);
                } else if (isFileSearchItem(item)) {
                  item.action();
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
                display: 'flex',
                alignItems: 'center',
                gap: 10,
              }}
            >
              {(() => {
                const icon = getIconForItem(item.name);
                if (icon) {
                  return <><img src={icon} alt="" style={{ width: 20, height: 20 }} />{item.name}</>;
                }
                return item.name;
              })()}
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