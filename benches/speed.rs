use std::hash::BuildHasher;
use std::hint::black_box;
use std::time::Duration;

use bloomfilter::Bloom;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use rapidhash::fast::SeedableState;
use uuid::Uuid;
use xorf::{BinaryFuse32, Xor32};

const MIN_ELE: usize = 102_400;
const MAX_ELE: usize = 1_024_000;
const STEP_ELE: usize = 102_400;

fn speed_benchmark(c: &mut Criterion) {
    let hasher = SeedableState::fixed();
    let keys = (0..MAX_ELE)
        .map(|_| Uuid::now_v7())
        .map(|uuid| hasher.hash_one(uuid))
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("build_filters");
    for ele in (MIN_ELE..=MAX_ELE).step_by(STEP_ELE) {
        let keys = &keys[..ele];
        group.throughput(Throughput::Elements(ele as u64));

        group.bench_with_input(BenchmarkId::new("BinaryFuse32", ele), keys, |b, keys| {
            b.iter(|| {
                let filter = BinaryFuse32::try_from(black_box(keys))
                    .expect("BinaryFuse32 construction failed (duplicate keys?)");
                black_box(filter);
            })
        });

        group.bench_with_input(BenchmarkId::new("Xor32", ele), keys, |b, keys| {
            b.iter(|| {
                let filter = Xor32::from(black_box(keys));
                black_box(filter);
            })
        });

        group.bench_with_input(BenchmarkId::new("Bloomfilter", ele), keys, |b, keys| {
            b.iter(|| {
                let mut filter =
                    Bloom::new_for_fp_rate(keys.len(), 0.0000000002328306436538696).unwrap();

                black_box(keys).iter().for_each(|it| filter.set(it));

                black_box(filter);
            })
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(6))
        .sample_size(10);
    targets = speed_benchmark
}
criterion_main!(benches);
