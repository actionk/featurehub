<script lang="ts">
  let {
    content,
    class: className = "",
  }: {
    content: string;
    class?: string;
  } = $props();

  interface Token {
    type: string;
    text: string;
  }

  function tokenize(source: string): Token[][] {
    const lines = source.split("\n");
    return lines.map((line) => tokenizeLine(line));
  }

  function tokenizeLine(line: string): Token[] {
    const tokens: Token[] = [];
    let remaining = line;

    // Check for comment
    const commentMatch = remaining.match(/^(\s*)#(.*)/);
    if (commentMatch) {
      if (commentMatch[1]) tokens.push({ type: "plain", text: commentMatch[1] });
      tokens.push({ type: "comment", text: "#" + commentMatch[2] });
      return tokens;
    }

    // Inside condition body — treat as CEL
    // We detect condition bodies by checking if the line is inside braces,
    // but since we tokenize line-by-line, we handle CEL keywords inline

    while (remaining.length > 0) {
      // Leading whitespace
      const ws = remaining.match(/^(\s+)/);
      if (ws) {
        tokens.push({ type: "plain", text: ws[1] });
        remaining = remaining.slice(ws[1].length);
        continue;
      }

      // Model keywords
      const modelKw = remaining.match(/^(model|schema|module)\b/);
      if (modelKw) {
        tokens.push({ type: "keyword-model", text: modelKw[1] });
        remaining = remaining.slice(modelKw[1].length);
        continue;
      }

      // Schema version
      const version = remaining.match(/^(\d+\.\d+)\b/);
      if (version) {
        tokens.push({ type: "version", text: version[1] });
        remaining = remaining.slice(version[1].length);
        continue;
      }

      // Type/structure keywords
      const typeKw = remaining.match(/^(type|relations|define|extend|condition)\b/);
      if (typeKw) {
        tokens.push({ type: "keyword-type", text: typeKw[1] });
        remaining = remaining.slice(typeKw[1].length);
        continue;
      }

      // "but not" as a single keyword
      const butNot = remaining.match(/^(but not)\b/);
      if (butNot) {
        tokens.push({ type: "keyword-operator", text: butNot[1] });
        remaining = remaining.slice(butNot[1].length);
        continue;
      }

      // Operator keywords
      const opKw = remaining.match(/^(and|or|from|with|self|as)\b/);
      if (opKw) {
        tokens.push({ type: "keyword-operator", text: opKw[1] });
        remaining = remaining.slice(opKw[1].length);
        continue;
      }

      // Condition parameter types
      const paramType = remaining.match(/^(bool|string|int|uint|double|duration|timestamp|ipaddress|map|list|any)\b/);
      if (paramType) {
        tokens.push({ type: "type-name", text: paramType[1] });
        remaining = remaining.slice(paramType[1].length);
        continue;
      }

      // CEL literals
      const celLiteral = remaining.match(/^(true|false|null)\b/);
      if (celLiteral) {
        tokens.push({ type: "literal", text: celLiteral[1] });
        remaining = remaining.slice(celLiteral[1].length);
        continue;
      }

      // Brackets and delimiters
      const bracket = remaining.match(/^([\[\]{}(),:.])/);
      if (bracket) {
        tokens.push({ type: "bracket", text: bracket[1] });
        remaining = remaining.slice(1);
        continue;
      }

      // CEL operators
      const celOp = remaining.match(/^(==|!=|<=|>=|&&|\|\||[+\-*/%<>!?])/);
      if (celOp) {
        tokens.push({ type: "operator", text: celOp[1] });
        remaining = remaining.slice(celOp[1].length);
        continue;
      }

      // Wildcard (user:*)
      const wildcard = remaining.match(/^(\*)/);
      if (wildcard) {
        tokens.push({ type: "wildcard", text: "*" });
        remaining = remaining.slice(1);
        continue;
      }

      // Type#relation reference
      const typeRef = remaining.match(/^([a-zA-Z_][a-zA-Z0-9_-]*)#([a-zA-Z_][a-zA-Z0-9_-]*)/);
      if (typeRef) {
        tokens.push({ type: "type-ref", text: typeRef[1] });
        tokens.push({ type: "bracket", text: "#" });
        tokens.push({ type: "relation-ref", text: typeRef[2] });
        remaining = remaining.slice(typeRef[0].length);
        continue;
      }

      // Identifiers
      const ident = remaining.match(/^([a-zA-Z_][a-zA-Z0-9_/-]*)/);
      if (ident) {
        tokens.push({ type: "identifier", text: ident[1] });
        remaining = remaining.slice(ident[1].length);
        continue;
      }

      // Numbers
      const num = remaining.match(/^(\d+)/);
      if (num) {
        tokens.push({ type: "literal", text: num[1] });
        remaining = remaining.slice(num[1].length);
        continue;
      }

      // Anything else
      tokens.push({ type: "plain", text: remaining[0] });
      remaining = remaining.slice(1);
    }

    return tokens;
  }

  let tokenizedLines = $derived(tokenize(content));
</script>

<div class="openfga-preview glass-panel--soft {className}">
  <pre class="openfga-code"><code>{#each tokenizedLines as tokens, i}{#if i > 0}{"\n"}{/if}{#each tokens as token}<span class="fga-{token.type}">{token.text}</span>{/each}{/each}</code></pre>
</div>
