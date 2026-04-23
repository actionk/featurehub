<script lang="ts">
  import { onMount } from "svelte";
  import { Terminal, type ILinkProvider, type ILink } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { WebLinksAddon } from "@xterm/addon-web-links";
  import { Unicode11Addon } from "@xterm/addon-unicode11";
  import { SearchAddon } from "@xterm/addon-search";
  import { open } from "@tauri-apps/plugin-shell";
  import { openPath } from "../../api/repos";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { ptyWrite, ptyResize } from "../../api/tauri";
  import { updateStatus, updateLabel } from "../../stores/terminals.svelte";
  import { getTerminalFontSize } from "../../stores/settings.svelte";
  import "@xterm/xterm/css/xterm.css";

  let {
    terminalId,
    active = false,
    onExited,
  }: {
    terminalId: string;
    active?: boolean;
    onExited?: () => void;
  } = $props();

  $effect(() => {
    const size = getTerminalFontSize() ?? 13;
    if (term && term.options.fontSize !== size) {
      term.options.fontSize = size;
      if (fitAddon && containerEl && containerEl.offsetWidth >= 10) {
        try {
          fitAddon.fit();
          ptyResize(terminalId, term.cols, term.rows).catch(() => {});
        } catch {}
      }
    }
  });

  $effect(() => {
    if (active && fitAddon && term) {
      // After visibility:hidden → visible transition, the container may not have
      // final dimensions on the first frame. Retry until we get reasonable cols.
      let attempts = 0;
      function tryFitActive() {
        if (!fitAddon || !term || !containerEl) return;
        // Skip if container is hidden (display:none ancestor gives zero dimensions)
        if (containerEl.offsetWidth < 10) return;
        // Preserve scroll position across fit using xterm's internal API
        const viewportY = term!.buffer.active.viewportY;
        fitAddon.fit();
        term!.scrollToLine(viewportY);
        if (term.cols <= 10 && attempts < 10) {
          attempts++;
          requestAnimationFrame(tryFitActive);
        } else {
          ptyResize(terminalId, term.cols, term.rows).catch(() => {});
          // Force xterm to repaint — canvas renderer can drop frames while the
          // container was visibility:hidden, leaving a black screen on return.
          try { term!.refresh(0, term!.rows - 1); } catch {}
          term!.focus();
        }
      }
      requestAnimationFrame(tryFitActive);
    }
  });

  // Handle file drag-and-drop: insert file path(s) into the terminal
  $effect(() => {
    if (!containerEl) return;
    const el = containerEl;
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        const paths = event.payload.paths;
        const pos = event.payload.position;
        const rect = el.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        const inBounds =
          (pos.x >= rect.left && pos.x <= rect.right &&
           pos.y >= rect.top && pos.y <= rect.bottom) ||
          (pos.x / dpr >= rect.left && pos.x / dpr <= rect.right &&
           pos.y / dpr >= rect.top && pos.y / dpr <= rect.bottom);
        if (inBounds && paths.length > 0) {
          const text = paths
            .map((p: string) => (p.includes(" ") ? `"${p}"` : p))
            .join(" ");
          const encoded = btoa(text);
          ptyWrite(terminalId, encoded).catch(() => {});
        }
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  });

  const SEARCH_DECO = {
    matchBackground: "#f0b23244",
    matchBorder: "#f0b232",
    matchOverviewRuler: "#f0b232",
    selectedMatchBackground: "#f0b23288",
    selectedMatchBorder: "#f0b232",
    selectedMatchOverviewRuler: "#f0b232",
  };

  let containerEl: HTMLDivElement | undefined = $state();
  let searchInputEl: HTMLInputElement | undefined = $state();
  let searchVisible = $state(false);
  let searchQuery = $state("");
  let term: Terminal | undefined;
  let fitAddon: FitAddon | undefined;
  let searchAddon: SearchAddon | undefined;
  let unlistenData: (() => void) | undefined;
  let unlistenExit: (() => void) | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let statusTimer: ReturnType<typeof setTimeout> | undefined;

  // Rolling buffer of recent raw text (last ~4KB, stripped of ANSI)
  let rawBuffer = "";
  const RAW_BUFFER_MAX = 4096;
  let lastNeedsInput = false;

  // Strip ANSI escape sequences and control chars from raw PTY output
  function stripAnsi(str: string): string {
    return str
      .replace(/\x1b\[[0-9;?]*[a-zA-Z]/g, "")   // CSI sequences
      .replace(/\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)/g, "") // OSC sequences
      .replace(/\x1b[()][A-Z0-9]/g, "")           // charset
      .replace(/\x1b[=><=]/g, "")                  // keypad mode
      .replace(/\x1b\[?[0-9;]*[hl]/g, "")         // mode set/reset
      .replace(/[\x00-\x08\x0b\x0e-\x1f\x7f]/g, ""); // control chars except \t \n \r \f
  }

  function processRawData(data: string) {
    const clean = stripAnsi(data);
    rawBuffer += clean;
    if (rawBuffer.length > RAW_BUFFER_MAX) {
      rawBuffer = rawBuffer.slice(-RAW_BUFFER_MAX);
    }
    scheduleStatusUpdate();
  }

  function extractStatus() {
    // Split by newlines and carriage returns, find last non-empty lines
    const lines = rawBuffer.split(/[\r\n]+/);
    const recentLines: string[] = [];

    // Collect last 3 meaningful lines for context
    for (let i = lines.length - 1; i >= 0 && recentLines.length < 3; i--) {
      const line = lines[i].trim();
      if (line.length === 0) continue;
      if (/^[─━│┃┌┐└┘├┤┬┴┼╭╮╰╯═╔╗╚╝╠╣╦╩╬\s\-|+]+$/.test(line)) continue;
      if (line.length < 3) continue;
      recentLines.push(line);
    }

    if (recentLines.length === 0) return;

    const statusLine = recentLines[0];
    // Only use immediate neighbors for context (not old buffer text)
    const context = recentLines.slice(0, 3).join(" ");

    // Detect BLOCKING input prompts only — not Claude's normal chat prompt
    const needsInput =
      // Claude Code edit/tool confirmations (check statusLine or immediate context)
      /Do you want to make this edit/i.test(context)
      || /Do you want to/i.test(statusLine)
      || /Esc to cancel/i.test(statusLine)
      || /Tab to amend/i.test(statusLine)
      // Yes/No prompts on the current line
      || /\(y\/n\)/i.test(statusLine)
      || /\(yes\/no\)/i.test(statusLine)
      || /\[Y\/n\]/i.test(statusLine)
      // Numbered choice menus (> 1. Yes)
      || /^>\s*\d+\.\s/.test(statusLine)
      // Direct question on the status line
      || /[?]\s*$/.test(statusLine)
      // Generic
      || /press enter/i.test(statusLine);

    // When transitioning from input-needed to not, trim buffer to avoid stale matches
    if (lastNeedsInput && !needsInput) {
      rawBuffer = rawBuffer.slice(-512);
    }
    lastNeedsInput = needsInput;

    updateStatus(terminalId, statusLine.slice(0, 120), needsInput);
  }

  function scheduleStatusUpdate() {
    if (statusTimer) clearTimeout(statusTimer);
    statusTimer = setTimeout(() => {
      statusTimer = undefined;
      extractStatus();
    }, 300);
  }

  onMount(() => {
    if (!containerEl) return;

    term = new Terminal({
      allowProposedApi: true,
      fontFamily: "'JetBrainsMono Nerd Font', 'JetBrains Mono', monospace",
      fontSize: getTerminalFontSize() ?? 13,
      lineHeight: 1.2,
      scrollback: 50000,
      cursorBlink: true,
      cursorInactiveStyle: 'none',
      theme: {
        background: "#0e0e12",
        foreground: "#e8e8ed",
        cursor: "#e8e8ed",
        selectionBackground: "rgba(124,124,255,0.3)",
        black: "#16161d",
        red: "#e5484d",
        green: "#3dd68c",
        yellow: "#f0b232",
        blue: "#52a9ff",
        magenta: "#a78bfa",
        cyan: "#22d3ee",
        white: "#e8e8ed",
        brightBlack: "#55556a",
        brightRed: "#e5484d",
        brightGreen: "#3dd68c",
        brightYellow: "#f0b232",
        brightBlue: "#52a9ff",
        brightMagenta: "#a78bfa",
        brightCyan: "#22d3ee",
        brightWhite: "#ffffff",
      },
    });

    fitAddon = new FitAddon();
    searchAddon = new SearchAddon();
    const unicode11Addon = new Unicode11Addon();
    term.loadAddon(fitAddon);
    term.loadAddon(searchAddon);
    term.loadAddon(unicode11Addon);
    term.loadAddon(new WebLinksAddon((_e, uri) => open(uri)));
    term.open(containerEl);
    term.unicode.activeVersion = "11";

    // File path link provider (ctrl+click to open in default app)
    // Matches Windows (C:\...) and Unix (/...) absolute paths
    const PATH_RE = /[A-Za-z]:[\\\/][^\s"'`<>|*?\r\n]+|\/[^\s"'`<>|*?\r\n]{2,}/g;
    const filePathProvider: ILinkProvider = {
      provideLinks(y: number, callback: (links: ILink[] | undefined) => void) {
        const line = term!.buffer.active.getLine(y - 1);
        if (!line) { callback(undefined); return; }
        const text = line.translateToString(true);
        const links: ILink[] = [];
        PATH_RE.lastIndex = 0;
        let m: RegExpExecArray | null;
        while ((m = PATH_RE.exec(text)) !== null) {
          // Skip paths that are part of a URL (preceded by : or /)
          const before = text[m.index - 1];
          if (before === ":" || before === "/") continue;
          // Trim trailing punctuation
          const raw = m[0].replace(/[.,;:)\]'"]+$/, "");
          if (raw.length < 2) continue;
          const startX = m.index + 1;
          const endX = startX + raw.length - 1;
          links.push({
            range: { start: { x: startX, y }, end: { x: endX, y } },
            text: raw,
            activate(_e: MouseEvent, text: string) {
              openPath(text).catch(() => {});
            },
          });
        }
        callback(links);
      },
    };
    term.registerLinkProvider(filePathProvider);

    term.onTitleChange((title) => {
      // Ignore generic titles that Claude Code sends via OSC sequences
      if (title && title !== "Claude Code") {
        updateLabel(terminalId, title);
      }
    });

    // Fit with retry: container may not have final dimensions on first paint
    let fitAttempts = 0;
    function tryFit() {
      if (!fitAddon || !term || !containerEl) return;
      // Skip if container is hidden (display:none ancestor gives zero dimensions)
      if (containerEl.offsetWidth < 10) {
        if (fitAttempts < 20) { fitAttempts++; setTimeout(tryFit, 50); }
        return;
      }
      fitAddon.fit();
      if ((term.rows <= 2 || term.cols <= 10) && fitAttempts < 10) {
        fitAttempts++;
        setTimeout(tryFit, 50);
      } else if (term.rows > 2) {
        ptyResize(terminalId, term.cols, term.rows).catch(() => {});
      }
    }
    requestAnimationFrame(tryFit);

    // Listen for PTY data
    listen<string>(`pty-data-${terminalId}`, (event) => {
      if (term) {
        const raw = atob(event.payload);
        const bytes = Uint8Array.from(raw, (c) => c.charCodeAt(0));
        term.write(bytes);
        // Decode as UTF-8 for status extraction (atob gives binary, not UTF-8)
        const utf8Text = new TextDecoder().decode(bytes);
        processRawData(utf8Text);
      }
    }).then((fn) => (unlistenData = fn));

    // Listen for PTY exit
    listen(`pty-exit-${terminalId}`, () => {
      term?.write("\r\n\x1b[90m[Process exited]\x1b[0m\r\n");
      onExited?.();
    }).then((fn) => (unlistenExit = fn));

    // Handle Ctrl+C (copy when selection exists, otherwise send SIGINT)
    // and Ctrl+V (paste from clipboard)
    term.attachCustomKeyEventHandler((e: KeyboardEvent) => {
      if (e.type !== "keydown") return true;

      if (e.ctrlKey && e.key === "f") {
        e.preventDefault();
        searchVisible = !searchVisible;
        if (searchVisible) setTimeout(() => searchInputEl?.focus(), 0);
        else term!.focus();
        return false;
      }

      if (e.key === "Escape" && searchVisible) {
        searchVisible = false;
        searchAddon?.clearDecorations();
        term!.focus();
        return false;
      }

      if (e.ctrlKey && e.key === "c" && term!.hasSelection()) {
        navigator.clipboard.writeText(term!.getSelection());
        term!.clearSelection();
        return false;
      }

      if (e.ctrlKey && e.key === "v") {
        e.preventDefault();
        navigator.clipboard.readText().then((text) => {
          if (text) {
            const bytes = new TextEncoder().encode(text);
            const encoded = btoa(String.fromCharCode(...bytes));
            ptyWrite(terminalId, encoded).catch(() => {});
          }
        });
        return false;
      }

      return true;
    });

    // Send input to PTY (encode as base64; use TextEncoder for Unicode support)
    term.onData((data) => {
      const bytes = new TextEncoder().encode(data);
      const encoded = btoa(String.fromCharCode(...bytes));
      ptyWrite(terminalId, encoded).catch(() => {});
    });

    // Handle binary data (e.g. from paste) — already Latin-1, btoa is safe
    term.onBinary((data) => {
      const encoded = btoa(data);
      ptyWrite(terminalId, encoded).catch(() => {});
    });

    // Resize handling — skip when container is hidden (display:none ancestor
    // causes zero dimensions; fitting then would send tiny cols to the PTY)
    resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0];
      if (!entry || entry.contentRect.width < 10 || entry.contentRect.height < 10) return;
      requestAnimationFrame(() => {
        if (fitAddon && term && containerEl && containerEl.offsetWidth > 10) {
          fitAddon.fit();
          ptyResize(terminalId, term.cols, term.rows).catch(() => {});
        }
      });
    });
    resizeObserver.observe(containerEl);

    return () => {
      if (statusTimer) clearTimeout(statusTimer);
      resizeObserver?.disconnect();
      unlistenData?.();
      unlistenExit?.();
      term?.dispose();
    };
  });
</script>

<div class="terminal-wrapper terminal">
  {#if searchVisible}
    <div class="terminal-search">
      <input
        bind:this={searchInputEl}
        bind:value={searchQuery}
        class="terminal-search-input"
        placeholder="Search…"
        onkeydown={(e) => {
          if (e.key === "Enter") {
            if (e.shiftKey) searchAddon?.findPrevious(searchQuery, { decorations: SEARCH_DECO });
            else searchAddon?.findNext(searchQuery, { decorations: SEARCH_DECO });
          }
          if (e.key === "Escape") {
            searchVisible = false;
            searchAddon?.clearDecorations();
            term?.focus();
          }
        }}
        oninput={() => {
          if (searchQuery) searchAddon?.findNext(searchQuery, { decorations: SEARCH_DECO, incremental: true });
          else searchAddon?.clearDecorations();
        }}
      />
      <button class="terminal-search-btn" onclick={() => searchAddon?.findPrevious(searchQuery, { decorations: SEARCH_DECO })} title="Previous (Shift+Enter)">↑</button>
      <button class="terminal-search-btn" onclick={() => searchAddon?.findNext(searchQuery, { decorations: SEARCH_DECO })} title="Next (Enter)">↓</button>
      <button class="terminal-search-btn terminal-search-close" onclick={() => { searchVisible = false; searchAddon?.clearDecorations(); term?.focus(); }}>✕</button>
    </div>
  {/if}
  <div class="terminal-container" bind:this={containerEl}></div>
</div>

<style>
  .terminal-search {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--bg-2, #1a1a24);
    border: 1px solid var(--border, #2a2a3a);
    border-radius: 6px;
    padding: 4px 6px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.4);
  }

  .terminal-search-input {
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-1, #e8e8ed);
    font-family: inherit;
    font-size: 13px;
    width: 200px;
    padding: 0 4px;
  }

  .terminal-search-input::placeholder {
    color: var(--text-3, #55556a);
  }

  .terminal-search-btn {
    background: transparent;
    border: none;
    color: var(--text-2, #a0a0b8);
    cursor: pointer;
    padding: 2px 5px;
    border-radius: 3px;
    font-size: 12px;
    line-height: 1;
  }

  .terminal-search-btn:hover {
    background: var(--bg-3, #2a2a3a);
    color: var(--text-1, #e8e8ed);
  }

  .terminal-search-close {
    margin-left: 2px;
  }
</style>

