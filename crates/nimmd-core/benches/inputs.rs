//! Входы бенчмарка и порога скорости (спек: рендер 1 МБ ≤ 100 мс): фрагмент повторяется до мебибайта.

use nimmd_core::FileKind;

const MIB: usize = 1024 * 1024;

const PROSE: &str = "# Заголовок раздела

Абзац с **выделением**, *курсивом*, `кодом` и [ссылкой](https://example.com/page).
Вторая строка абзаца с ещё одной [ссылкой](docs/readme.md) и ~~зачёркнутым~~ текстом.

## Подраздел

- пункт списка с **жирным**
- пункт со ссылкой [сюда](#anchor)
- третий пункт

| Колонка | Значение | Примечание |
| --- | ---: | --- |
| альфа | 1,5 | первая |
| бета | 22 | вторая |

```rust
fn main() {
    let xs: Vec<u32> = (0..10).map(|x| x * 2).collect();
    println!(\"{xs:?}\");
}
```

";

const CODE: &str = "```rust
pub fn sum(xs: &[i64]) -> i64 {
    xs.iter().copied().fold(0, |a, b| a.saturating_add(b))
}
```

```python
def fib(n: int) -> int:
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
```

```js
export function debounce(fn, ms) {
  let t;
  return (...args) => { clearTimeout(t); t = setTimeout(() => fn(...args), ms); };
}
```

```bash
for f in *.md; do
  wc -l \"$f\" | awk '{print $1}'
done
```

";

const JSON_ITEM: &str = "  {\"id\": 12345, \"name\": \"Позиция\", \"price\": 12.5, \"tags\": [\"a\", \"b\"], \"active\": true},\n";

const CSV_ROW: &str = "12345;Позиция номер один;12,50;3;37,50\n";

/// Фрагмент, повторённый до длины не меньше мебибайта.
fn repeat(fragment: &str) -> String {
    fragment.repeat(MIB.div_ceil(fragment.len()))
}

/// Четыре входа: имя, тип файла, текст не короче мебибайта.
pub fn inputs() -> Vec<(&'static str, FileKind, String)> {
    let json = format!("[\n{}  {{\"id\": 0}}\n]\n", repeat(JSON_ITEM));
    let csv = format!("id;name;price;qty;total\n{}", repeat(CSV_ROW));
    vec![
        ("prose", FileKind::Markdown, repeat(PROSE)),
        ("code", FileKind::Markdown, repeat(CODE)),
        ("json", FileKind::Json, json),
        ("csv", FileKind::Csv, csv),
    ]
}
