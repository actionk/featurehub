<script lang="ts">
  import DOMPurify from "dompurify";
  import type { Marked } from "marked";
  import { tick } from "svelte";
  import { getMermaidEnabled, getOpenFgaEnabled } from "../stores/settings.svelte";

  // Lazy-load marked to reduce initial bundle size
  let MarkedClass: typeof Marked | null = null;
  let markedLoadPromise: Promise<typeof Marked> | null = null;
  async function loadMarked(): Promise<typeof Marked> {
    if (MarkedClass) return MarkedClass;
    if (!markedLoadPromise) {
      markedLoadPromise = import("marked").then(m => {
        MarkedClass = m.Marked;
        return m.Marked;
      });
    }
    return markedLoadPromise;
  }

  let {
    content,
    class: className = "",
  }: {
    content: string;
    class?: string;
  } = $props();

  let previewEl: HTMLDivElement | undefined = $state();

  // Marked instance is created lazily after the library loads

  // Store raw mermaid sources keyed by index so we don't lose them to HTML encoding
  let mermaidSources: string[] = [];

  const mermaidExtension = {
    name: "mermaidBlock",
    level: "block" as const,
    // Match ```mermaid ... ``` fenced code blocks
    start(src: string) {
      return src.match(/^```mermaid/m)?.index;
    },
    tokenizer(src: string) {
      const match = src.match(/^```mermaid\s*\n([\s\S]*?)```/);
      if (match) {
        return {
          type: "mermaidBlock",
          raw: match[0],
          text: match[1].trim(),
        };
      }
      return undefined;
    },
    renderer(token: { text: string }) {
      const idx = mermaidSources.length;
      mermaidSources.push(token.text);
      // Render as a placeholder div with a data attribute pointing to the source index
      return `<div class="mermaid-placeholder" data-mermaid-idx="${idx}"></div>`;
    },
  };

  const openfgaExtension = {
    name: "openfgaBlock",
    level: "block" as const,
    start(src: string) {
      return src.match(/^```(?:openfga|fga)/m)?.index;
    },
    tokenizer(src: string) {
      const match = src.match(/^```(?:openfga|fga)\s*\n([\s\S]*?)```/);
      if (match) {
        return {
          type: "openfgaBlock",
          raw: match[0],
          text: match[1].trim(),
        };
      }
      return undefined;
    },
    renderer(token: { text: string }) {
      return `<pre class="openfga-code"><code>${highlightOpenFga(token.text)}</code></pre>`;
    },
  };

  function highlightOpenFga(source: string): string {
    return source.split("\n").map(highlightOpenFgaLine).join("\n");
  }

  function highlightOpenFgaLine(line: string): string {
    // Comments
    const commentMatch = line.match(/^(\s*)#(.*)/);
    if (commentMatch) {
      return `${escapeHtml(commentMatch[1])}<span class="fga-comment">#${escapeHtml(commentMatch[2])}</span>`;
    }

    let result = "";
    let remaining = line;

    while (remaining.length > 0) {
      // Leading whitespace
      const ws = remaining.match(/^(\s+)/);
      if (ws) {
        result += ws[1];
        remaining = remaining.slice(ws[1].length);
        continue;
      }

      // Model keywords
      const modelKw = remaining.match(/^(model|schema|module)\b/);
      if (modelKw) {
        result += `<span class="fga-keyword-model">${modelKw[1]}</span>`;
        remaining = remaining.slice(modelKw[1].length);
        continue;
      }

      // Schema version
      const version = remaining.match(/^(\d+\.\d+)\b/);
      if (version) {
        result += `<span class="fga-version">${version[1]}</span>`;
        remaining = remaining.slice(version[1].length);
        continue;
      }

      // Type/structure keywords
      const typeKw = remaining.match(/^(type|relations|define|extend|condition)\b/);
      if (typeKw) {
        result += `<span class="fga-keyword-type">${typeKw[1]}</span>`;
        remaining = remaining.slice(typeKw[1].length);
        continue;
      }

      // "but not"
      const butNot = remaining.match(/^(but not)\b/);
      if (butNot) {
        result += `<span class="fga-keyword-operator">${butNot[1]}</span>`;
        remaining = remaining.slice(butNot[1].length);
        continue;
      }

      // Operator keywords
      const opKw = remaining.match(/^(and|or|from|with|self|as)\b/);
      if (opKw) {
        result += `<span class="fga-keyword-operator">${opKw[1]}</span>`;
        remaining = remaining.slice(opKw[1].length);
        continue;
      }

      // Condition parameter types
      const paramType = remaining.match(/^(bool|string|int|uint|double|duration|timestamp|ipaddress|map|list|any)\b/);
      if (paramType) {
        result += `<span class="fga-type-name">${paramType[1]}</span>`;
        remaining = remaining.slice(paramType[1].length);
        continue;
      }

      // CEL literals
      const celLiteral = remaining.match(/^(true|false|null)\b/);
      if (celLiteral) {
        result += `<span class="fga-literal">${celLiteral[1]}</span>`;
        remaining = remaining.slice(celLiteral[1].length);
        continue;
      }

      // Brackets/delimiters
      const bracket = remaining.match(/^([\[\]{}(),:.])/);
      if (bracket) {
        result += `<span class="fga-bracket">${escapeHtml(bracket[1])}</span>`;
        remaining = remaining.slice(1);
        continue;
      }

      // CEL operators
      const celOp = remaining.match(/^(==|!=|<=|>=|&&|\|\||[+\-*/%<>!?])/);
      if (celOp) {
        result += `<span class="fga-operator">${escapeHtml(celOp[1])}</span>`;
        remaining = remaining.slice(celOp[1].length);
        continue;
      }

      // Type#relation reference
      const typeRef = remaining.match(/^([a-zA-Z_][a-zA-Z0-9_-]*)#([a-zA-Z_][a-zA-Z0-9_-]*)/);
      if (typeRef) {
        result += `<span class="fga-type-ref">${escapeHtml(typeRef[1])}</span><span class="fga-bracket">#</span><span class="fga-relation-ref">${escapeHtml(typeRef[2])}</span>`;
        remaining = remaining.slice(typeRef[0].length);
        continue;
      }

      // Identifiers
      const ident = remaining.match(/^([a-zA-Z_][a-zA-Z0-9_/-]*)/);
      if (ident) {
        result += `<span class="fga-identifier">${escapeHtml(ident[1])}</span>`;
        remaining = remaining.slice(ident[1].length);
        continue;
      }

      // Numbers
      const num = remaining.match(/^(\d+)/);
      if (num) {
        result += `<span class="fga-literal">${num[1]}</span>`;
        remaining = remaining.slice(num[1].length);
        continue;
      }

      // Anything else
      result += escapeHtml(remaining[0]);
      remaining = remaining.slice(1);
    }

    return result;
  }

  let mermaidEnabled = $derived(getMermaidEnabled());
  let openfgaEnabled = $derived(getOpenFgaEnabled());

  function buildMarkedInstance(MC: typeof Marked, mermaid: boolean, openfga: boolean): InstanceType<typeof Marked> {
    const instance = new MC();
    const exts: typeof mermaidExtension[] = [];
    if (mermaid) exts.push(mermaidExtension);
    if (openfga) exts.push(openfgaExtension);
    if (exts.length > 0) instance.use({ extensions: exts });
    return instance;
  }

  let markedReady = $state(false);
  let activeMarked = $state<InstanceType<typeof Marked> | null>(null);

  // Load marked on mount and rebuild when settings change
  $effect(() => {
    const mermaid = mermaidEnabled;
    const openfga = openfgaEnabled;
    loadMarked().then(MC => {
      activeMarked = buildMarkedInstance(MC, mermaid, openfga);
      markedReady = true;
    });
  });

  let rawHtml = $derived.by(() => {
    if (!activeMarked) return escapeHtml(content);
    mermaidSources = [];
    return activeMarked.parse(content) as string;
  });

  $effect(() => {
    const _html = rawHtml;
    const enabled = mermaidEnabled;
    const el = previewEl;
    if (!el || !enabled) return;

    const sources = [...mermaidSources];
    tick().then(() => renderMermaidBlocks(el, sources));
  });

  let mermaidLoaded = false;

  async function renderMermaidBlocks(el: HTMLDivElement, sources: string[]) {
    const placeholders = el.querySelectorAll(".mermaid-placeholder");
    if (placeholders.length === 0) return;

    const { default: mermaid } = await import("mermaid");
    if (!mermaidLoaded) {
      mermaid.initialize({
        startOnLoad: false,
        theme: "dark",
        themeVariables: {
          darkMode: true,
          background: "#1e1e2e",
          primaryColor: "#a78bfa",
          primaryTextColor: "#e0e0e0",
          primaryBorderColor: "#6d5dbb",
          lineColor: "#888",
          secondaryColor: "#3b3b5c",
          tertiaryColor: "#2a2a40",
        },
      });
      mermaidLoaded = true;
    }

    for (const el of placeholders) {
      const placeholder = el as HTMLElement;
      if (placeholder.dataset.mermaidRendered === "true") continue;

      const idx = parseInt(placeholder.dataset.mermaidIdx ?? "-1", 10);
      const source = sources[idx];
      if (!source) continue;

      try {
        const id = `mermaid-${crypto.randomUUID().slice(0, 8)}`;
        const { svg } = await mermaid.render(id, source);
        placeholder.className = "mermaid-diagram";
        placeholder.innerHTML = svg;
        placeholder.dataset.mermaidRendered = "true";
      } catch (err) {
        console.warn("Mermaid render error:", err);
        placeholder.className = "";
        placeholder.innerHTML = `<pre class="mermaid-error"><code>${escapeHtml(source)}</code></pre>`;
        placeholder.dataset.mermaidRendered = "true";
      }
    }
  }

  function escapeHtml(str: string): string {
    return str
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  let safeHtml = $derived(DOMPurify.sanitize(rawHtml, { ADD_ATTR: ['class'], FORBID_TAGS: ['style'] }));
</script>

<div class="markdown-body markdown-preview {className}" bind:this={previewEl}>
  {@html safeHtml}
</div>
