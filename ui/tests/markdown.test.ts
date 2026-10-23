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
  it('excerpt is short plain text', () => {
    expect(excerpt('## Head\n- **bold** item\n- [link](https://x)')).toBe('Head bold item link');
    expect(excerpt('a'.repeat(300), 10)).toHaveLength(10);
    expect(excerpt(null)).toBe('');
  });
});
