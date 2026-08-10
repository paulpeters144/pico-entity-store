use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use pico_entity_store::prelude::*;

const ENTITY_COUNTS: &[usize] = &[1000, 10000];
const BATCH_REMOVE_COUNT: usize = 100;
const SAMPLE_SIZE: usize = 100;
const DESCENDANT_CHAIN_DEPTH: usize = 100;

#[derive(Clone)]
#[allow(dead_code)]
struct BenchmarkEntity {
    x: u64,
    y: u64,
    z: u64,
}

#[derive(Clone)]
#[allow(dead_code)]
struct OtherBenchmarkEntity {
    x: u64,
    y: u64,
    z: u64,
}

fn add_entities_mixed(store: &EntityStore, n: usize) {
    for i in 0..n {
        if i % 2 == 0 {
            store.add(BenchmarkEntity { x: i as u64, y: 0, z: 0 }, &[]).unwrap();
        } else {
            store.add(OtherBenchmarkEntity { x: i as u64, y: 0, z: 0 }, &[]).unwrap();
        }
    }
}

fn add_entities_single(store: &EntityStore, n: usize) {
    for i in 0..n {
        store.add(BenchmarkEntity { x: i as u64, y: 0, z: 0 }, &[]).unwrap();
    }
}

// ── add/bulk (mixed types, matches C# Add) ────────────────────────────────

fn bench_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    for &entity_count in ENTITY_COUNTS {
        group.bench_with_input(BenchmarkId::new("bulk", entity_count), &entity_count, |b, &n| {
            b.iter_batched(
                || (),
                |_| {
                    let store = EntityStore::new();
                    add_entities_mixed(&store, n);
                    store
                },
                BatchSize::SmallInput,
            );
        });
    }

    for &entity_count in ENTITY_COUNTS {
        group.bench_with_input(BenchmarkId::new("batch", entity_count), &entity_count, |b, &n| {
            b.iter_batched(
                || {
                    let entities: Vec<BenchmarkEntity> =
                        (0..n).map(|i| BenchmarkEntity { x: i as u64, y: 0, z: 0 }).collect();
                    entities
                },
                |entities| {
                    let store = EntityStore::new();
                    for entity in entities {
                        store.add(entity, &[]).unwrap();
                    }
                    store
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── add/marginal (single add into a pre-grown, warm-capacity store) ──────

fn bench_add_marginal(c: &mut Criterion) {
    let mut group = c.benchmark_group("add");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    // `Vec::clear` retains capacity, so after this warm-up + clear every
    // refill and every timed add below reuses already-allocated storage —
    // no realloc or page-fault churn inside the timed loop.
    let store = EntityStore::new();
    add_entities_single(&store, 10_000);
    store.clear();

    group.bench_function("marginal_10k", |b| {
        b.iter_custom(|iters| {
            // Untimed reset: back to exactly 10k live entities.
            store.clear();
            add_entities_single(&store, 10_000);
            let start = std::time::Instant::now();
            for i in 0..iters {
                store.add(BenchmarkEntity { x: i, y: 0, z: 0 }, &[]).unwrap();
            }
            start.elapsed()
        });
    });

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
        let last_id = store.first::<BenchmarkEntity>().unwrap().id();

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

// ── all/sum (mixed store, sum field by type — matches C# All(typeof(T))) ─

fn bench_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("all");
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(1));
    group.sample_size(SAMPLE_SIZE);

    for &entity_count in ENTITY_COUNTS {
        let store = EntityStore::new();
        add_entities_mixed(&store, entity_count);

        group.bench_with_input(
            BenchmarkId::new("sum", entity_count),
            &entity_count,
            |b, _n| {
                b.iter(|| {
                    black_box(
                        store
                            .all::<BenchmarkEntity>()
                            .fold(0u64, |acc, e| acc.wrapping_add(e.x)),
                    );
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

        group.bench_with_input(BenchmarkId::new("first", entity_count), &entity_count, |b, _n| {
            b.iter(|| {
                black_box(store.first::<BenchmarkEntity>());
            });
        });
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
        store.add(BenchmarkEntity { x: i as u64, y: 0, z: 0 }, &[]).unwrap();
    }
    for i in 0..DESCENDANT_CHAIN_DEPTH as u64 {
        let parent = store.get_by_id::<BenchmarkEntity>(i).unwrap();
        let child = store.get_by_id::<BenchmarkEntity>(i + 1).unwrap();
        store.add(parent, &children![child]).unwrap();
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
                        let erefs: Vec<EntityRef> = (0..BATCH_REMOVE_COUNT as u64)
                            .map(|id| EntityRef::from_raw(
                                id,
                                std::any::TypeId::of::<BenchmarkEntity>(),
                            ))
                            .collect();
                        store.remove(&erefs);
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
                let mut parent_erefs = Vec::new();
                for _ in 0..BATCH_REMOVE_COUNT {
                    let p_ref = store.add(BenchmarkEntity { x: 0, y: 0, z: 0 }, &[]).unwrap();
                    parent_erefs.push(p_ref);
                    let child_erefs: Vec<ChildSource> = (0..10)
                        .map(|_| store.add(BenchmarkEntity { x: 0, y: 0, z: 0 }, &[]).unwrap())
                        .map(ChildSource::Existing)
                        .collect();
                    let parent = store.get_by_id::<BenchmarkEntity>(p_ref.id()).unwrap();
                    store.add(parent, &child_erefs).unwrap();
                }
                (store, parent_erefs)
            },
            |(store, parent_erefs)| {
                store.remove(&parent_erefs);
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
    bench_add_marginal,
    bench_get_by_id,
    bench_all,
    bench_first,
    bench_descendants,
    bench_remove,
    bench_remove_with_children
);
criterion_main!(benches);
