// Рендер формул KaTeX: текст формулы берётся из textContent, HTML документа не трогается
document.addEventListener('DOMContentLoaded', () => {
  for (const el of document.querySelectorAll('[data-math-style]')) {
    katex.render(el.textContent, el, {
      displayMode: el.dataset.mathStyle === 'display',
      trust: false,
      throwOnError: false,
    });
  }
});
