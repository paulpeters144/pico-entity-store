use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use pico_ecs::prelude::EntityStore;

#[allow(dead_code)]
#[derive(Clone)]
struct BenchmarkEntity {
    x: u64,
    y: u64,
    z: u64,
}

#[allow(dead_code)]
#[derive(Clone)]
struct OtherBenchmarkEntity {
    a: u64,
    b: u64,
}

fn bench_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(20);

    for entity_count in [100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("bulk", entity_count),
            &entity_count,
            |b, &n| {
                b.iter_batched(
                    || (),
                    |_| {
                        let store = EntityStore::new();
                        for i in 0..n {
                            if i % 2 == 0 {
                                store.add(&BenchmarkEntity { x: i as u64, y: 0, z: 0 });
                            } else {
                                store.add(&OtherBenchmarkEntity { a: i as u64, b: 0 });
                            }
                        }
                        store
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn bench_get_by_id(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_by_id");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(20);

    for entity_count in [100, 1000] {
        let store = EntityStore::new();
        let mut last_id = 0u64;
        for i in 0..entity_count {
            store.add(&BenchmarkEntity { x: i as u64, y: 0, z: 0 });
            last_id = i as u64;
        }

        group.bench_with_input(
            BenchmarkId::new("single_lookup", entity_count),
            &last_id,
            |b, &id| {
                b.iter(|| {
                    black_box(store.get_by_id::<BenchmarkEntity>(black_box(id)));
                });
            },
        );
    }

    group.finish();
}

fn bench_each(c: &mut Criterion) {
    let mut group = c.benchmark_group("each");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(20);

    for entity_count in [100, 1000] {
        let store = EntityStore::new();
        for i in 0..entity_count {
            if i % 2 == 0 {
                store.add(&BenchmarkEntity { x: i as u64, y: 0, z: 0 });
            } else {
                store.add(&OtherBenchmarkEntity { a: i as u64, b: 0 });
            }
        }

        group.bench_with_input(
            BenchmarkId::new("iterate", entity_count),
            &entity_count,
            |b, _n| {
                b.iter(|| {
                    store.each::<BenchmarkEntity, _>(|e| {
                        black_box(e);
                    });
                });
            },
        );
    }

    group.finish();
}

fn bench_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(20);

    let remove_count = 100;

    group.bench_function("batch_remove", |b| {
        b.iter_batched(
            || {
                let store = EntityStore::new();
                for i in 0..remove_count {
                    store.add(&BenchmarkEntity { x: i as u64, y: 0, z: 0 });
                }
                store
            },
            |store| {
                let ids: Vec<u64> =
                    store.all::<BenchmarkEntity>().map(|r| r.id()).collect();
                for id in ids {
                    store.remove_by_id(id);
                }
                store
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_add, bench_get_by_id, bench_each, bench_remove);
criterion_main!(benches);
