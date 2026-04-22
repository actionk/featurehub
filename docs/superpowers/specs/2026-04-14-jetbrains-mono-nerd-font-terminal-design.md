# JetBrains Mono Nerd Font — Terminal Bundling

**Date:** 2026-04-14

## Problem

The terminal uses `@fontsource/jetbrains-mono`, which does not include Nerd Font glyph patches. Claude Code's status bar outputs powerline separators and icon glyphs (U+E0B0 range and others) that render as `?` boxes.

## Goal

Bundle JetBrains Mono Nerd Font with the app so the terminal renders all Nerd Font glyphs correctly without requiring the user to install anything.

## Approach

Bundle `.ttf` files directly in `src/assets/fonts/`. Vite picks them up via `@font-face` `url()` references in CSS and includes them in the production build. No backend changes needed.

## Files Added

```
src/assets/fonts/
  JetBrainsMonoNerdFont-Regular.ttf
  JetBrainsMonoNerdFont-Bold.ttf
  JetBrainsMonoNerdFont-Italic.ttf
  JetBrainsMonoNerdFont-BoldItalic.ttf
```

Source: `ryanoasis/nerd-fonts` GitHub releases, latest v3.x stable. Use the "Mono" variant (single-width glyphs) — correct for xterm.js cell rendering.

## CSS Change (`src/app.css`)

Add four `@font-face` declarations at the top of the file:

```css
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 400; font-style: normal;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Regular.ttf')    format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 700; font-style: normal;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Bold.ttf')       format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 400; font-style: italic;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Italic.ttf')     format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 700; font-style: italic;  src: url('./assets/fonts/JetBrainsMonoNerdFont-BoldItalic.ttf') format('truetype'); }
```

## Terminal Change (`src/lib/modules/ai/Terminal.svelte`)

Line 188, change `fontFamily`:

```js
fontFamily: "'JetBrainsMono Nerd Font', 'JetBrains Mono', monospace",
```

Fallback chain ensures graceful degradation if font fails to load.

## Out of Scope

- UI code font (`@fontsource/jetbrains-mono`) — unchanged
- Tauri config — no changes needed
- Settings system — no changes needed
- Rust backend — no changes needed
