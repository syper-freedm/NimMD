//! Порог скорости рендера (спек: 1 МБ ≤ 100 мс на Apple Silicon): `just bench-check`.

#[path = "../benches/inputs.rs"]
mod inputs;

use std::env;
use std::hint::black_box;
use std::time::{Duration, Instant};

use nimmd_core::{RenderOptions, render};

const BUDGET: Duration = Duration::from_millis(100);

/// Множитель порога из `NIMMD_BENCH_FACTOR`; нет или не число — 1.
fn factor() -> u32 {
    env::var("NIMMD_BENCH_FACTOR")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1)
}

#[test]
#[ignore = "порог скорости в release: just bench-check"]
fn render_one_mib_within_budget() {
    let budget = BUDGET * factor();
    let mut medians = Vec::new();
    for (name, kind, src) in inputs::inputs() {
        let opts = RenderOptions {
            kind,
            ..RenderOptions::default()
        };
        // Прогрев: ленивые грамматики syntect и регулярные выражения
        black_box(render(black_box(&src), &opts));
        let mut times: Vec<Duration> = (0..5)
            .map(|_| {
                let start = Instant::now();
                black_box(render(black_box(&src), &opts));
                start.elapsed()
            })
            .collect();
        times.sort_unstable();
        let median = times[2];
        println!("{name}: медиана {median:?}, порог {budget:?}");
        medians.push((name, median));
    }
    for (name, median) in medians {
        assert!(
            median <= budget,
            "{name}: медиана {median:?} больше порога {budget:?}"
        );
    }
}
