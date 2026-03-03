use std::hash::BuildHasher;
use std::hint::black_box;
use std::time::Duration;

use bloomfilter::Bloom;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rapidhash::fast::SeedableState;
use uuid::Uuid;
use xorf::{BinaryFuse32, Filter, Xor32};

const MIN_ELE: usize = 102_400;
const MAX_ELE: usize = 1_024_000;
const STEP_ELE: usize = 102_400;
const QUERY_COUNT: usize = 4_096;

fn contains_hit_benchmark(c: &mut Criterion) {
    let hasher = SeedableState::fixed();
    let keys_all = (0..MAX_ELE)
        .map(|_| Uuid::now_v7())
        .map(|uuid| hasher.hash_one(uuid))
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("contains_hit");

    for ele in (MIN_ELE..=MAX_ELE).step_by(STEP_ELE) {
        let keys = &keys_all[..ele];
        let queries = (0..QUERY_COUNT)
            .map(|i| {
                let idx = i.wrapping_mul(2_654_435_761) % ele;
                keys[idx]
            })
            .collect::<Vec<_>>();

        let binary_fuse_filter = BinaryFuse32::try_from(keys)
            .expect("BinaryFuse32 construction failed (duplicate keys?)");
        group.bench_with_input(
            BenchmarkId::new("BinaryFuse32", ele),
            &queries,
            |b, queries| {
                b.iter(|| {
                    let mut hit_count = 0u64;
                    for query in queries {
                        hit_count += binary_fuse_filter.contains(black_box(query)) as u64;
                    }
                    black_box(hit_count);
                })
            },
        );

        let xor_filter = Xor32::from(keys);
        group.bench_with_input(BenchmarkId::new("Xor32", ele), &queries, |b, queries| {
            b.iter(|| {
                let mut hit_count = 0u64;
                for query in queries {
                    hit_count += xor_filter.contains(black_box(query)) as u64;
                }
                black_box(hit_count);
            })
        });

        let mut bloom_filter =
            Bloom::new_for_fp_rate(keys.len(), 0.0000000002328306436538696).unwrap();
        keys.iter().for_each(|it| bloom_filter.set(it));
        group.bench_with_input(
            BenchmarkId::new("Bloomfilter", ele),
            &queries,
            |b, queries| {
                b.iter(|| {
                    let mut hit_count = 0u64;
                    for query in queries {
                        hit_count += bloom_filter.check(black_box(query)) as u64;
                    }
                    black_box(hit_count);
                })
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(6))
        .sample_size(10);
    targets = contains_hit_benchmark
}
criterion_main!(benches);
