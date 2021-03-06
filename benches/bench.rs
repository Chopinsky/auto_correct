#![allow(dead_code)]

#[macro_use]
extern crate criterion;
extern crate auto_correct;

use criterion::Criterion;
use auto_correct::prelude::*;

fn bench_base(c: &mut Criterion) {
    let mut service = AutoCorrect::new();
    service.set_max_edit(2);

    c.bench_function("auto_correct: 'tets'", move |b| {
        b.iter(|| {
            let results = service.candidates(String::from("tets"));
            assert_eq!(results.len(), 367usize);
        })
    });
}

fn bench_long(c: &mut Criterion) {
    let mut service = AutoCorrect::new();
    service.set_max_edit(2);

    c.bench_function("auto_correct: 'wahtabout'", move |b| {
        b.iter(|| {
            let results = service.candidates(String::from("wahtabout"));
            assert_eq!(results.len(), 1usize);
        })
    });
}

criterion_group!(benches, bench_base); //, bench_long);
criterion_main!(benches);