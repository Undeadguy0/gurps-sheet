// Minimal, safe Markdown renderer for explanations (no dependencies).
// All source text is HTML-escaped FIRST; only a fixed set of tags is emitted,
// raw HTML in the source is shown as text. Links: http(s)/mailto only.
// Supported: # headings, paragraphs, line breaks, - / * / 1. lists, > quotes,
// ``` code blocks, `code`, **bold**, *italic* / _italic_, [text](url), ---,
// nested lists (indent by 2+ spaces), GFM-style | tables | with :--: alignment.

const esc = (s: string) =>
  s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');

function inline(src: string): string {
  const codes: string[] = [];
  let s = esc(src).replace(/`([^`]+)`/g, (_, c) => `\u0000${codes.push(c) - 1}\u0000`);
  s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_m, text, url) => {
    const u = url.replace(/&amp;/g, '&');
    if (!/^(https?:\/\/|mailto:)/i.test(u)) return text;
    return `<a href="${esc(u)}" target="_blank" rel="noopener noreferrer">${text}</a>`;
  });
  s = s.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>').replace(/__([^_]+)__/g, '<strong>$1</strong>');
  s = s.replace(/(^|[^*])\*([^*\s][^*]*)\*/g, '$1<em>$2</em>').replace(/(^|[^\w_])_([^_\s][^_]*)_(?!\w)/g, '$1<em>$2</em>');
  return s.replace(/\u0000(\d+)\u0000/g, (_, i) => `<code>${codes[+i]}</code>`);
}

const LIST_RE = /^(\s*)(?:([-*+])|(\d+)[.)])\s+(.*)$/;
const SEP_RE = /^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/;
const cells = (l: string) => l.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map((c) => c.trim());

interface Item { text: string; sub: List[] }
interface List { tag: 'ul' | 'ol'; indent: number; items: Item[] }
const renderList = (l: List): string =>
  `<${l.tag}>${l.items.map((i) => `<li>${inline(i.text)}${i.sub.map(renderList).join('')}</li>`).join('')}</${l.tag}>`;

/** Consecutive list lines -> nested lists by indentation. */
function parseLists(lines: string[]): List[] {
  const roots: List[] = [];
  const stack: List[] = [];
  for (const line of lines) {
    const m = line.match(LIST_RE)!;
    const indent = m[1].replace(/\t/g, '    ').length; const tag = m[2] ? 'ul' : 'ol';
    while (stack.length && indent < stack[stack.length - 1].indent) stack.pop();
    let top = stack[stack.length - 1];
    if (top && indent > top.indent && top.items.length) {
      const sub: List = { tag, indent, items: [] };
      top.items[top.items.length - 1].sub.push(sub); stack.push(sub); top = sub;
    } else if (!top || top.tag !== tag) {
      const l: List = { tag, indent, items: [] };
      if (top) stack.pop();
      const parent = stack[stack.length - 1];
      if (parent?.items.length) parent.items[parent.items.length - 1].sub.push(l); else roots.push(l);
      stack.push(l); top = l;
    }
    top.items.push({ text: m[4], sub: [] });
  }
  return roots;
}

function renderTable(rows: string[]): string {
  const align = cells(rows[1]).map((c) => (/^:-+:$/.test(c) ? 'center' : /-:$/.test(c) ? 'right' : /^:-/.test(c) ? 'left' : ''));
  const td = (tag: string, c: string, i: number) => `<${tag}${align[i] ? ` style="text-align:${align[i]}"` : ''}>${inline(c)}</${tag}>`;
  const head = cells(rows[0]);
  const body = rows.slice(2).map((r) => { const c = cells(r); return `<tr>${head.map((_, i) => td('td', c[i] ?? '', i)).join('')}</tr>`; });
  return `<table><thead><tr>${head.map((c, i) => td('th', c, i)).join('')}</tr></thead>${body.length ? `<tbody>${body.join('')}</tbody>` : ''}</table>`;
}

export function renderMarkdown(md: string | null | undefined): string {
  if (!md) return '';
  const lines = md.replace(/\r\n?/g, '\n').split('\n');
  const out: string[] = [];
  let para: string[] = [];
  const flush = () => { if (para.length) out.push(`<p>${para.map(inline).join('<br>')}</p>`); para = []; };
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    let m: RegExpMatchArray | null;
    if (/^\s*```/.test(line)) {
      flush();
      const code: string[] = [];
      while (++i < lines.length && !/^\s*```/.test(lines[i])) code.push(lines[i]);
      out.push(`<pre><code>${esc(code.join('\n'))}</code></pre>`);
    } else if (!line.trim()) flush();
    else if (line.includes('|') && i + 1 < lines.length && lines[i + 1].includes('-') && SEP_RE.test(lines[i + 1])) {
      flush();
      const rows = [line, lines[++i]];
      while (i + 1 < lines.length && lines[i + 1].includes('|') && lines[i + 1].trim()) rows.push(lines[++i]);
      out.push(renderTable(rows));
    }
    else if ((m = line.match(/^\s*(#{1,6})\s+(.*)$/))) { flush(); const n = Math.min(6, m[1].length + 3); out.push(`<h${n}>${inline(m[2])}</h${n}>`); }
    else if (/^\s*([-*_])(\s*\1){2,}\s*$/.test(line)) { flush(); out.push('<hr>'); }
    else if ((m = line.match(/^\s*>\s?(.*)$/))) { flush(); out.push(`<blockquote>${inline(m[1])}</blockquote>`); }
    else if (LIST_RE.test(line)) {
      flush();
      const block = [line];
      while (i + 1 < lines.length && LIST_RE.test(lines[i + 1])) block.push(lines[++i]);
      out.push(parseLists(block).map(renderList).join(''));
    } else para.push(line.trim());
  }
  flush();
  return out.join('');
}

/** Plain-text excerpt for tooltips: Markdown syntax stripped, whitespace collapsed. */
export function excerpt(md: string | null | undefined, max = 160): string {
  if (!md) return '';
  const t = md
    .replace(/```[\s\S]*?(```|$)/g, ' ')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/^\s*(#{1,6}|>|[-*+]|\d+[.)])\s+/gm, '')
    .replace(/[*_`]+/g, '')
    .replace(/\s+/g, ' ')
    .trim();
  return t.length > max ? `${t.slice(0, max - 1).trimEnd()}…` : t;
}
