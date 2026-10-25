// Диаграммы Mermaid: elkjs не завендорен, поэтому layout elk заменяется на dagre
document.addEventListener('DOMContentLoaded', () => {
  const blocks = document.querySelectorAll('pre.mermaid');
  for (const el of blocks) {
    el.textContent = el.textContent.replace(/layout:\s*elk/g, 'layout: dagre');
  }
  const theme = document.documentElement.dataset.theme;
  const dark = theme === 'dark' ||
    (theme !== 'light' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  mermaid.initialize({
    startOnLoad: false,
    layout: 'dagre',
    securityLevel: 'strict',
    theme: dark ? 'dark' : 'default',
  });
  mermaid.run({ querySelector: 'pre.mermaid' });
});
