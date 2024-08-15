use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memguard_rs::{ct_eq, Secret, Zeroize};

fn bench_zeroize_slice(c: &mut Criterion) {
    let mut buf = vec![0x42u8; 4096];
    c.bench_function("zeroize_slice/4096", |bencher| {
        bencher.iter(|| {
            black_box(&mut buf).zeroize();
        })
    });

    let mut buf_small = vec![0x42u8; 32];
    c.bench_function("zeroize_slice/32", |bencher| {
        bencher.iter(|| {
            black_box(&mut buf_small).zeroize();
        })
    });
}

fn bench_ct_eq(c: &mut Criterion) {
    let a = vec![0x42u8; 32];
    let b = vec![0x42u8; 32];
    c.bench_function("ct_eq/32", |bencher| {
        bencher.iter(|| black_box(ct_eq(black_box(&a), black_box(&b))))
    });

    let a_large = vec![0x42u8; 4096];
    let b_large = vec![0x42u8; 4096];
    c.bench_function("ct_eq/4096", |bencher| {
        bencher.iter(|| black_box(ct_eq(black_box(&a_large), black_box(&b_large))))
    });
}

fn bench_secret_drop(c: &mut Criterion) {
    c.bench_function("secret_drop/32", |bencher| {
        bencher.iter(|| {
            let _s = Secret::new(black_box([0x42u8; 32]));
        })
    });
}

criterion_group!(benches, bench_zeroize_slice, bench_ct_eq, bench_secret_drop);
criterion_main!(benches);
// 2024-08-19 — Fix benchmark compilation on stable Rust
// 2024-11-04 — Add test for ct_select_slice with mismatched lengths (should panic)
// 2025-03-13 — Add test for GuardedRegion::from_bytes with maximum N=4096
// 2025-06-23 — Add benchmark for GuardedRegion creation and drop overhead
// 2025-10-06 — Refactor mlock tests to use helper for optional lock assertions
// 2026-01-24 — Fix broken intra-doc links flagged by rustdoc --check
// 2026-05-02 — Add test for zeroize on Vec<u8> with capacity > len
// 2026-05-15 — Add test for ct_eq_full with zero-length inputs
// 2026-07-10 — Add test for GuardedRegion::from_bytes with N=1
