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
