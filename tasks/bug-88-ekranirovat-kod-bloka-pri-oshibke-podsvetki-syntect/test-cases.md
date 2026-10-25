# Тестовые сценарии: Экранировать код блока при ошибке подсветки syntect

## Метаданные
- Issue: #88
- Стек: `cargo test` (insta для снапшотов), static — `git grep`/`cargo tree`/`just lint`. Ролей и разграничения доступа у библиотеки нет — сценарии доступа неприменимы.

## Покрытие
| Критерий | Сценарии |
| --- | --- |
| AC-1 | SC-001 |
| AC-2 | SC-002, SC-003 |
| AC-3 | SC-004, SC-005 |
| AC-4 | SC-006, SC-007 |

## Сценарии
### SC-001: Err подсветки → экранированный текст целиком
- **Покрывает:** AC-1
- **Уровень:** unit
- **Предусловия:** `#[cfg(test)]`-модуль в `render.rs` (функция FR-2 не публичная).
- **Шаги:** вызвать функцию FR-2 с `Err(syntect::Error::Fmt(fmt::Error))` и текстом `<script>alert(1)</script> & "x"`.
- **Ожидание:** вывод `== "&lt;script&gt;alert(1)&lt;/script&gt; &amp; &quot;x&quot;"`; нет `<script`, `<span`.
- **Результат выполнения:** pass

### SC-002: Ok подсветки пишется без изменений
- **Покрывает:** AC-2
- **Уровень:** unit
- **Предусловия:** как SC-001.
- **Шаги:** вызвать функцию FR-2 с `Ok("<span class=\"source\">x</span>")`.
- **Ожидание:** вывод равен строке `Ok` побайтно.
- **Результат выполнения:** pass

### SC-003: Снапшот и frontmatter не меняются
- **Покрывает:** AC-2
- **Уровень:** integration
- **Предусловия:** правка применена, снапшоты не перезаписывались.
- **Шаги:** `cargo test -p nimmd-core --test render code_highlighted_with_classes`; `cargo test -p nimmd-core --test document` (в т. ч. `bom_crlf_frontmatter_and_title`, `frontmatter_text_is_escaped`, `not_utf8_banner_first_in_main`); `git diff --exit-code crates/nimmd-core/tests/snapshots/`.
- **Ожидание:** тесты зелёные, diff снапшотов пуст, файлов `*.snap.new` нет.
- **Результат выполнения:** pass

### SC-004: Неизвестный язык — экранирован, язык в разметку не попадает
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** —
- **Шаги:** `render_fragment("```nosuchlang\n<script>x</script>\n```\n")`.
- **Ожидание:** содержит `<pre class="syntax-highlighting"><code>` и `&lt;script&gt;`; нет `<script`, `nosuchlang`.
- **Результат выполнения:** pass

### SC-005: Блок без языка — экранирован, `<code>` без класса
- **Покрывает:** AC-3
- **Уровень:** integration
- **Предусловия:** —
- **Шаги:** `render_fragment("```\n<img src=x onerror=y>\n```\n")`.
- **Ожидание:** содержит `<pre class="syntax-highlighting"><code>` и `&lt;img`; нет `<img`.
- **Результат выполнения:** pass

### SC-006: SyntectAdapter убран, syntect один
- **Покрывает:** AC-4
- **Уровень:** static
- **Предусловия:** —
- **Шаги:** `git grep -n SyntectAdapter crates/`; `cargo tree -p nimmd-core -d -e normal`; `git grep -n syntect Cargo.toml crates/nimmd-core/Cargo.toml`; `git grep -c 'name = "syntect"' Cargo.lock`.
- **Ожидание:** grep пуст; `syntect` нет среди дублей; зависимость с `default-features = false` и `default-syntaxes`, `html`, `regex-fancy`; в `Cargo.lock` одна запись `5.3.x`.
- **Результат выполнения:** pass

### SC-007: Линтеры и тесты рабочего пространства
- **Покрывает:** AC-4
- **Уровень:** integration
- **Предусловия:** установлены инструменты `just lint`, есть сеть (cargo deny).
- **Шаги:** `just lint`; `cargo test --workspace --all-targets`.
- **Ожидание:** оба завершаются с кодом 0.
- **Результат выполнения:** pass
