import { describe, expect, it } from 'vitest';
import { excerpt, renderMarkdown } from '../src/markdown';

describe('markdown', () => {
  it('renders basic syntax', () => {
    const h = renderMarkdown('# Title\nline *one*\nline **two**\n\n- a\n- `b`\n\n1. x');
    expect(h).toBe('<h4>Title</h4><p>line <em>one</em><br>line <strong>two</strong></p><ul><li>a</li><li><code>b</code></li></ul><ol><li>x</li></ol>');
  });
  it('escapes raw HTML and unsafe links', () => {
    const h = renderMarkdown('<img src=x onerror=alert(1)> [x](javascript:alert(1)) [ok](https://e.org/?a=1&b=2)\n```\n<script>\n```');
    expect(h).not.toMatch(/<img|<script|javascript:/);
    expect(h).toContain('&lt;img');
    expect(h).toContain('<a href="https://e.org/?a=1&amp;b=2" target="_blank" rel="noopener noreferrer">ok</a>');
    expect(h).toContain('<pre><code>&lt;script&gt;</code></pre>');
  });
  it('renders nested lists', () => {
    expect(renderMarkdown('- a\n  - b\n    1. c\n  - d\n- e')).toBe(
      '<ul><li>a<ul><li>b<ol><li>c</li></ol></li><li>d</li></ul></li><li>e</li></ul>');
    expect(renderMarkdown('- a\n1. b')).toBe('<ul><li>a</li></ul><ol><li>b</li></ol>');
    expect(renderMarkdown('   - over-indented first')).toBe('<ul><li>over-indented first</li></ul>');
  });
  it('renders tables with alignment and escapes cells', () => {
    const h = renderMarkdown('| Кто | Долг |\n|:--|--:|\n| гном | **5** |\n| <b>x</b> |');
    expect(h).toBe('<table><thead><tr><th style="text-align:left">Кто</th><th style="text-align:right">Долг</th></tr></thead>'
      + '<tbody><tr><td style="text-align:left">гном</td><td style="text-align:right"><strong>5</strong></td></tr>'
      + '<tr><td style="text-align:left">&lt;b&gt;x&lt;/b&gt;</td><td style="text-align:right"></td></tr></tbody></table>');
    expect(renderMarkdown('a | b\nno separator')).toBe('<p>a | b<br>no separator</p>');
  });
  it('excerpt is short plain text', () => {
    expect(excerpt('## Head\n- **bold** item\n- [link](https://x)')).toBe('Head bold item link');
    expect(excerpt('a'.repeat(300), 10)).toHaveLength(10);
    expect(excerpt(null)).toBe('');
  });
});
