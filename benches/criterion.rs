use criterion::BenchmarkId;
use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use std::fmt::Debug;

extern crate bt;
use bt::arena::Tree;

const K: usize = 256;

#[inline]
fn insert<T>(values: &Vec<T>)
where
    T: Ord + Copy + Default + Debug,
{
    let mut t = Tree::<T, K>::default();
    for &v in values {
        t.insert(v);
    }
}

#[inline]
fn insert_delete<T>(values: &Vec<T>, delete_values: &Vec<T>)
where
    T: Ord + Copy + Default + Debug,
{
    let mut t = Tree::<_, K>::default();
    for &v in values {
        t.insert(v);
    }

    for &v in delete_values {
        t.delete(v);
    }
}

#[inline]
fn rand_vec(n: u64, seed: usize) -> Vec<u64> {
    let mut rng = Pcg64::seed_from_u64(seed as u64);
    let mut vec: Vec<_> = (0..n).collect();
    vec.shuffle(&mut rng);
    return vec;
}

fn benchmark_rand_insert(b: &mut Bencher, n: u64, seed: usize) {
    let vec = rand_vec(n, seed);
    b.iter(|| insert(black_box(&vec)))
}

fn benchmark_rand_insert_delete_half(b: &mut Bencher, n: u64, seed: usize) {
    let vec = rand_vec(n, seed);
    let vec_to_delete = rand_vec(n / 2, seed + 1);
    b.iter(|| insert_delete(black_box(&vec), black_box(&vec_to_delete)))
}

fn benchmark_seq_insert(b: &mut Bencher, n: usize) {
    let vec: Vec<_> = (0_u64..n as u64).collect();
    b.iter(|| insert(&vec))
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("seq_insert");
    for size in [1_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &s| {
            benchmark_seq_insert(b, *s as usize);
        });
    }
    group.finish();

    const DEFAULT_SEED: usize = 1024;
    let mut group = c.benchmark_group("rand_insert");
    for size in [1_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &s| {
            benchmark_rand_insert(b, *s as u64, DEFAULT_SEED);
        });
    }
    group.finish();

    let mut group = c.benchmark_group("rand_insert_delete_half");
    for size in [1_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &s| {
            benchmark_rand_insert_delete_half(b, *s as u64, DEFAULT_SEED);
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
