# JetBrains Mono Nerd Font — Terminal Bundling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bundle JetBrains Mono Nerd Font with the app so the terminal renders Nerd Font glyphs (powerline separators, icons) correctly without requiring the user to install anything.

**Architecture:** Download four `.ttf` files (Regular, Bold, Italic, BoldItalic) from the ryanoasis/nerd-fonts GitHub release and place them in `src/assets/fonts/`. Declare them via `@font-face` in `app.css`. Update the xterm.js terminal `fontFamily` to use the new family name with a fallback.

**Tech Stack:** Svelte 5, xterm.js (`@xterm/xterm`), Vite (handles font bundling via CSS `url()` references), CSS `@font-face`

---

## File Map

| File | Change |
|------|--------|
| `src/assets/fonts/JetBrainsMonoNerdFont-Regular.ttf` | Create (download) |
| `src/assets/fonts/JetBrainsMonoNerdFont-Bold.ttf` | Create (download) |
| `src/assets/fonts/JetBrainsMonoNerdFont-Italic.ttf` | Create (download) |
| `src/assets/fonts/JetBrainsMonoNerdFont-BoldItalic.ttf` | Create (download) |
| `src/app.css` | Add four `@font-face` declarations at top |
| `src/lib/modules/ai/Terminal.svelte` | Change `fontFamily` on line 188 |

---

### Task 1: Download Nerd Font files

**Files:**
- Create: `src/assets/fonts/JetBrainsMonoNerdFont-Regular.ttf`
- Create: `src/assets/fonts/JetBrainsMonoNerdFont-Bold.ttf`
- Create: `src/assets/fonts/JetBrainsMonoNerdFont-Italic.ttf`
- Create: `src/assets/fonts/JetBrainsMonoNerdFont-BoldItalic.ttf`

- [ ] **Step 1: Create the fonts directory**

```bash
mkdir -p src/assets/fonts
```

- [ ] **Step 2: Download the JetBrainsMono Nerd Fonts zip**

```bash
curl -L "https://github.com/ryanoasis/nerd-fonts/releases/download/v3.3.0/JetBrainsMono.zip" -o /tmp/JetBrainsMono.zip
```

Expected: file `/tmp/JetBrainsMono.zip` created (~25MB).

- [ ] **Step 3: Extract only the four needed files**

```bash
cd /tmp && unzip -o JetBrainsMono.zip \
  "JetBrainsMonoNerdFont-Regular.ttf" \
  "JetBrainsMonoNerdFont-Bold.ttf" \
  "JetBrainsMonoNerdFont-Italic.ttf" \
  "JetBrainsMonoNerdFont-BoldItalic.ttf" \
  -d /tmp/jb_nf_extract
```

Expected: four `.ttf` files extracted to `/tmp/jb_nf_extract/`.

- [ ] **Step 4: Copy into the project**

Run from the repo root:

```bash
cp /tmp/jb_nf_extract/JetBrainsMonoNerdFont-Regular.ttf    src/assets/fonts/
cp /tmp/jb_nf_extract/JetBrainsMonoNerdFont-Bold.ttf       src/assets/fonts/
cp /tmp/jb_nf_extract/JetBrainsMonoNerdFont-Italic.ttf     src/assets/fonts/
cp /tmp/jb_nf_extract/JetBrainsMonoNerdFont-BoldItalic.ttf src/assets/fonts/
```

Expected: `ls src/assets/fonts/` shows four `.ttf` files.

- [ ] **Step 5: Commit**

```bash
git add src/assets/fonts/
git commit -m "feat(terminal): add JetBrainsMono Nerd Font files"
```

---

### Task 2: Register font via `@font-face` in CSS

**Files:**
- Modify: `src/app.css` (add four `@font-face` blocks before the existing `@import` lines)

- [ ] **Step 1: Add `@font-face` declarations at the top of `src/app.css`**

Open `src/app.css`. The file currently starts with:

```css
@import '@fontsource/space-grotesk/400.css';
```

Insert the following four lines **before** the existing `@import` statements (i.e., as the very first lines of the file):

```css
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 400; font-style: normal;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Regular.ttf')    format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 700; font-style: normal;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Bold.ttf')       format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 400; font-style: italic;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Italic.ttf')     format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 700; font-style: italic;  src: url('./assets/fonts/JetBrainsMonoNerdFont-BoldItalic.ttf') format('truetype'); }
```

The top of the file should now look like:

```css
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 400; font-style: normal;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Regular.ttf')    format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 700; font-style: normal;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Bold.ttf')       format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 400; font-style: italic;  src: url('./assets/fonts/JetBrainsMonoNerdFont-Italic.ttf')     format('truetype'); }
@font-face { font-family: 'JetBrainsMono Nerd Font'; font-weight: 700; font-style: italic;  src: url('./assets/fonts/JetBrainsMonoNerdFont-BoldItalic.ttf') format('truetype'); }
@import '@fontsource/space-grotesk/400.css';
@import '@fontsource/space-grotesk/500.css';
...
```

- [ ] **Step 2: Commit**

```bash
git add src/app.css
git commit -m "feat(terminal): register JetBrainsMono Nerd Font via @font-face"
```

---

### Task 3: Update terminal `fontFamily`

**Files:**
- Modify: `src/lib/modules/ai/Terminal.svelte:188`

- [ ] **Step 1: Change the `fontFamily` option in the `Terminal` constructor**

In `src/lib/modules/ai/Terminal.svelte`, find line 188 (inside `onMount`, inside `new Terminal({...})`):

```js
      fontFamily: "'JetBrains Mono', monospace",
```

Replace with:

```js
      fontFamily: "'JetBrainsMono Nerd Font', 'JetBrains Mono', monospace",
```

- [ ] **Step 2: Run the app and verify visually**

```bash
npm run tauri dev
```

Open a feature, open the terminal tab, start a Claude session. The Claude Code status bar at the bottom of the terminal should now show:
- Powerline triangle separators (``, ``) instead of `?` boxes
- Percentage signs and arrows render correctly
- All text remains sharp and correctly sized

If glyphs still show as `?` boxes, open browser devtools in the Tauri window (`Ctrl+Shift+I`), go to the Elements tab, inspect `.xterm-screen`, and confirm the computed `font-family` includes `'JetBrainsMono Nerd Font'`. If the font failed to load it will appear in the Network tab as a 404.

- [ ] **Step 3: Commit**

```bash
git add src/lib/modules/ai/Terminal.svelte
git commit -m "feat(terminal): use JetBrainsMono Nerd Font for Nerd Font glyph support"
```
