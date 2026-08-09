use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use pico_ecs::prelude::EntityStore;

const ENTITY_COUNTS: &[usize] = &[1000, 10000];
const BATCH_REMOVE_COUNT: usize = 100;
const SAMPLE_SIZE: usize = 100;
const DESCENDANT_CHAIN_DEPTH: usize = 100;

#[derive(Clone)]
struct BenchmarkEntity {
    x: u64,
    y: u64,
    z: u64,
}

#[derive(Clone)]
struct OtherBenchmarkEntity {
    x: u64,
    y: u64,
    z: u64,
}

fn add_entities_mixed(store: &EntityStore, n: usize) {
    for i in 0..n {
        if i % 2 == 0 {
            store.add(&BenchmarkEntity {
                x: i as u64,
                y: 0,
                z: 0,
            });
        } else {
            store.add(&OtherBenchmarkEntity {
                x: i as u64,
                y: 0,
                z: 0,
            });
        }
    }
}

fn add_entities_single(store: &EntityStore, n: usize) {
    for i in 0..n {
        store.add(&BenchmarkEntity {
            x: i as u64,
            y: 0,
            z: 0,
        });
    }
}

// ── add/bulk (mixed types, matches C# Add) ────────────────────────────────

fn bench_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    for &entity_count in ENTITY_COUNTS {
        group.bench_with_input(
            BenchmarkId::new("bulk", entity_count),
            &entity_count,
            |b, &n| {
                b.iter_batched(
                    || (),
                    |_| {
                        let store = EntityStore::new();
                        add_entities_mixed(&store, n);
                        store
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ── get_by_id (mixed store, lookup BenchmarkEntity) ───────────────────────

fn bench_get_by_id(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_by_id");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    for &entity_count in ENTITY_COUNTS {
        let store = EntityStore::new();
        add_entities_mixed(&store, entity_count);
        let last_id = store
            .all::<BenchmarkEntity>()
            .last()
            .expect("should have entities")
            .id();

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

// ── each/iterate (single type, matches C# All scope) ─────────────────────

fn bench_each(c: &mut Criterion) {
    let mut group = c.benchmark_group("each");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    for &entity_count in ENTITY_COUNTS {
        let store = EntityStore::new();
        add_entities_single(&store, entity_count);

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

// ── all/collect (mixed store, collect by type — matches C# All(typeof(T))) ─

fn bench_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("all");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    for &entity_count in ENTITY_COUNTS {
        let store = EntityStore::new();
        add_entities_mixed(&store, entity_count);

        group.bench_with_input(
            BenchmarkId::new("collect", entity_count),
            &entity_count,
            |b, _n| {
                b.iter(|| {
                    black_box(store.all::<BenchmarkEntity>().collect::<Vec<_>>());
                });
            },
        );
    }

    group.finish();
}

// ── first (mixed store, first of type — matches C# First<T>()) ───────────

fn bench_first(c: &mut Criterion) {
    let mut group = c.benchmark_group("first");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    for &entity_count in ENTITY_COUNTS {
        let store = EntityStore::new();
        add_entities_mixed(&store, entity_count);

        group.bench_with_input(
            BenchmarkId::new("first", entity_count),
            &entity_count,
            |b, _n| {
                b.iter(|| {
                    black_box(store.first::<BenchmarkEntity>());
                });
            },
        );
    }

    group.finish();
}

// ── descendants (100-deep chain, matches C# Descendants()) ───────────────

fn bench_descendants(c: &mut Criterion) {
    let mut group = c.benchmark_group("descendants");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    let store = EntityStore::new();
    for i in 0..=DESCENDANT_CHAIN_DEPTH {
        store.add(&BenchmarkEntity {
            x: i as u64,
            y: 0,
            z: 0,
        });
    }
    for i in 0..DESCENDANT_CHAIN_DEPTH as u64 {
        store.add_children_ids(i, &[i + 1]).unwrap();
    }

    let root_ref = store.get_by_id::<BenchmarkEntity>(0).unwrap();

    group.bench_function("chain_100", |b| {
        b.iter(|| {
            black_box(store.descendants(&root_ref));
        });
    });

    group.finish();
}

// ── remove/batch_remove (single type, 100 removals — matches C# Remove) ──

fn bench_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    for &entity_count in ENTITY_COUNTS {
        group.bench_with_input(
            BenchmarkId::new("batch_remove", entity_count),
            &entity_count,
            |b, &n| {
                b.iter_batched(
                    || {
                        let store = EntityStore::new();
                        add_entities_single(&store, n);
                        store
                    },
                    |store| {
                        let ids: Vec<u64> = store
                            .all::<BenchmarkEntity>()
                            .take(BATCH_REMOVE_COUNT)
                            .map(|r| r.id())
                            .collect();
                        for id in ids {
                            store.remove_by_id(id);
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

// ── remove_with_children (100 parents × 10 children, BFS removal) ────────

fn bench_remove_with_children(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_with_children");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    group.bench_function("batch_100_with_children", |b| {
        b.iter_batched(
            || {
                let store = EntityStore::new();
                let mut parent_ids = Vec::new();
                for _ in 0..BATCH_REMOVE_COUNT {
                    let p_id = store.count() as u64;
                    store.add(&BenchmarkEntity { x: 0, y: 0, z: 0 });
                    parent_ids.push(p_id);
                    let child_ids: Vec<u64> = (0..10)
                        .map(|_| {
                            let id = store.count() as u64;
                            store.add(&BenchmarkEntity { x: 0, y: 0, z: 0 });
                            id
                        })
                        .collect();
                    store.add_children_ids(p_id, &child_ids).unwrap();
                }
                (store, parent_ids)
            },
            |(store, parent_ids)| {
                for id in parent_ids {
                    store.remove_by_id(id);
                }
                store
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_add,
    bench_get_by_id,
    bench_each,
    bench_all,
    bench_first,
    bench_descendants,
    bench_remove,
    bench_remove_with_children
);
criterion_main!(benches);
