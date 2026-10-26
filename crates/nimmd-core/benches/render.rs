//! Бенчмарк `render` на входах от мебибайта (`just bench`).

mod inputs;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use nimmd_core::{RenderOptions, render};

fn render_inputs(c: &mut Criterion) {
    for (name, kind, src) in inputs::inputs() {
        let opts = RenderOptions {
            kind,
            ..RenderOptions::default()
        };
        let mut group = c.benchmark_group(name);
        // Рендер 1 МиБ длится десятки миллисекунд: 10 замеров хватает
        group.sample_size(10);
        group.bench_function("render", |b| b.iter(|| render(black_box(&src), &opts)));
        group.finish();
    }
}

criterion_group!(benches, render_inputs);
criterion_main!(benches);
