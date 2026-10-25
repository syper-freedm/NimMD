# Контракт CSS-классов NimMD

Классы, которые ядро `nimmd-core` выводит в HTML (спек 5.4, В6). Классовые селекторы
`nimmd.css` берутся только из этих списков; тест контракта сверяет их с выводом ядра.
Классы подсветки syntect (`highlight.css`) в контракт не входят.

## Общие с редактором

- `markdown-alert`
- `markdown-alert-title`
- `markdown-alert-note`
- `markdown-alert-tip`
- `markdown-alert-important`
- `markdown-alert-warning`
- `markdown-alert-caution`
- `contains-task-list`
- `task-list-item`
- `task-list-item-checkbox`
- `footnote-ref`
- `footnotes`
- `frontmatter`
- `mermaid`

## Только ядра

- `footnote-backref`
- `nimmd`
- `nimmd-data`
- `data`
- `banner`
- `truncated`
- `not-utf8`
- `remote-image`
