use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_pcg::Pcg64;

extern crate bt;
use bt::arena::Tree;

#[inline]
fn insert(values: &Vec<usize>) {
    let mut t = Tree::new(3);
    for &v in values {
        t.insert(v);
    }
}

#[inline]
fn insert_delete(values: &Vec<usize>, delete_values: &Vec<usize>) {
    let mut t = Tree::new(3);
    for &v in values {
        t.insert(v);
    }

    for &v in delete_values {
        t.delete(v);
    }
}

#[inline]
fn rand_vec(n: usize, seed: usize) -> Vec<usize> {
    let mut rng = Pcg64::seed_from_u64(seed as u64);
    let mut vec: Vec<_> = (0..n).collect();
    vec.shuffle(&mut rng);
    return vec;
}

fn benchmark_insert(b: &mut Bencher, n: usize, seed: usize) {
    let vec = rand_vec(n, seed);
    b.iter(|| insert(black_box(&vec)))
}

fn benchmark_insert_delete_half(b: &mut Bencher, n: usize, seed: usize) {
    let vec = rand_vec(n, seed);
    let vec_to_delete = rand_vec(n / 2, seed + 1);
    b.iter(|| insert_delete(black_box(&vec), black_box(&vec_to_delete)))
}

fn criterion_benchmark(c: &mut Criterion) {
    const DEFAULT_SEED: usize = 2;
    for size in [2, 4, 8, 16, 32, 64, 128, 256, 512, 1024] {
        c.bench_function(format!("insert {}", size).as_str(), |b| {
            benchmark_insert(b, size, DEFAULT_SEED)
        });

        c.bench_function(format!("insert delete_half {}", size).as_str(), |b| {
            benchmark_insert_delete_half(b, size, DEFAULT_SEED)
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
