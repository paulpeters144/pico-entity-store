use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
    health: i32,
}

#[derive(Clone)]
struct Axe {
    damage: u32,
    durability: u32,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Shield {
    defense: u32,
}

fn dwarf(name: &str, health: i32) -> Dwarf {
    Dwarf {
        name: name.into(),
        health,
    }
}

#[test]
fn add_increases_count() {
    let store = EntityStore::new();
    assert_eq!(store.count(), 0);
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    assert_eq!(store.count(), 1);
    store.add(dwarf("Thorin", 150), &[]).unwrap();
    assert_eq!(store.count(), 2);
}

#[test]
fn add_returns_new_entity_id() {
    let store = EntityStore::new();
    let id0 = store.add(dwarf("Gimli", 100), &[]).unwrap();
    let id1 = store.add(dwarf("Thorin", 150), &[]).unwrap();
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
}

#[test]
fn first_returns_ref_that_derefs_correctly() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    assert_eq!(d.name, "Gimli");
    assert_eq!(d.health, 100);
}

#[test]
fn ref_id_returns_consistent_value() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    let id1 = d.id();
    let id2 = d.id();
    assert_eq!(id1, id2);
}

#[test]
fn refmut_deref_mut_mutates_entity_in_store() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    {
        let mut d = store.first_mut::<Dwarf>().unwrap();
        d.health -= 30;
    }
    let d = store.first::<Dwarf>().unwrap();
    assert_eq!(d.health, 70);
}

#[test]
fn first_mut_returns_mutable_access() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    {
        let mut d = store.first_mut::<Dwarf>().unwrap();
        d.name = "Mutated".into();
    }
    let d = store.first::<Dwarf>().unwrap();
    assert_eq!(d.name, "Mutated");
}

#[test]
fn get_by_id_mut_returns_mutable_access() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let id = store.first::<Dwarf>().unwrap().id();
    {
        let mut d = store.get_by_id_mut::<Dwarf>(id).unwrap();
        d.health = 42;
    }
    let d = store.first::<Dwarf>().unwrap();
    assert_eq!(d.health, 42);
}

#[test]
fn each_mut_callback_mutates_all_entities() {
    let store = EntityStore::new();
    store
        .add(
            Axe {
                damage: 10,
                durability: 50,
            },
            &[],
        )
        .unwrap();
    store
        .add(
            Axe {
                damage: 20,
                durability: 60,
            },
            &[],
        )
        .unwrap();
    store.each_mut::<Axe, _>(|a| {
        a.durability -= 1;
        a.damage += 5;
    });
    let axes: Vec<_> = store.all::<Axe>().collect();
    assert_eq!(axes[0].damage, 15);
    assert_eq!(axes[0].durability, 49);
    assert_eq!(axes[1].damage, 25);
    assert_eq!(axes[1].durability, 59);
}

#[test]
fn update_updates_single_entity_by_entity_ref() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let ref_d = store.first::<Dwarf>().unwrap();
    let eref = ref_d.entity_ref();
    drop(ref_d);

    let updated = store.update::<Dwarf, _>(&eref, |d| {
        d.health -= 10;
    });
    assert!(updated);

    let d = store.first::<Dwarf>().unwrap();
    assert_eq!(d.health, 90);
}

#[test]
fn update_returns_false_for_wrong_type() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let ref_d = store.first::<Dwarf>().unwrap();
    let eref = ref_d.entity_ref();
    drop(ref_d);

    assert!(!store.update::<Axe, _>(&eref, |_| {}));
}

#[test]
fn each_callback_receives_all_entities_of_type() {
    let store = EntityStore::new();
    store.add(dwarf("A", 10), &[]).unwrap();
    store.add(dwarf("B", 20), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 5,
                durability: 5,
            },
            &[],
        )
        .unwrap();

    let mut names = Vec::new();
    store.each::<Dwarf, _>(|d| names.push(d.name.clone()));
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"A".to_string()));
    assert!(names.contains(&"B".to_string()));
}

#[test]
fn all_iterator_yields_all_entities() {
    let store = EntityStore::new();
    store.add(dwarf("A", 10), &[]).unwrap();
    store.add(dwarf("B", 20), &[]).unwrap();

    let dwarves: Vec<_> = store.all::<Dwarf>().collect();
    assert_eq!(dwarves.len(), 2);
    assert_eq!(dwarves[0].name, "A");
    assert_eq!(dwarves[1].name, "B");
}

#[test]
fn get_by_id_returns_correct_entity() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(dwarf("Thorin", 150), &[]).unwrap();

    let d1 = store.first::<Dwarf>().unwrap();
    let id = d1.id();
    drop(d1);

    let d = store.get_by_id::<Dwarf>(id).unwrap();
    assert_eq!(d.name, "Gimli");
}

#[test]
fn get_by_id_returns_none_for_wrong_id() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    assert!(store.get_by_id::<Dwarf>(999).is_none());
}

#[test]
fn attach_child_links_parent_child() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let a = store.first::<Axe>().unwrap();
    store.add(d, &children![a]).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let children = store.children(&d);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id(), store.first::<Axe>().unwrap().id());
}

#[test]
fn attach_child_via_refmut_parent() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(Shield { defense: 80 }, &[]).unwrap();

    // Collect the child EntityRef (dropping its read guard) before
    // acquiring the write guard for the parent.
    let shield = store.first::<Shield>().unwrap();
    let kids = children![shield];
    let parent = store.first_mut::<Dwarf>().unwrap();
    store.add(parent, &kids).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let children = store.children(&d);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id(), store.first::<Shield>().unwrap().id());
}

#[test]
fn add_new_entity_with_children_links_them() {
    let store = EntityStore::new();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();

    let a1 = store.first::<Axe>().unwrap();
    let parent_id = store.add(dwarf("Parent", 100), &children![a1]).unwrap();

    let d = store.get_by_id::<Dwarf>(parent_id).unwrap();
    let children = store.children(&d);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id(), store.first::<Axe>().unwrap().id());
}

#[test]
fn parent_returns_correct_entity_ref() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let a = store.first::<Axe>().unwrap();
    store.add(d, &children![a]).unwrap();

    let a = store.first::<Axe>().unwrap();
    let parent = store.parent(&a).unwrap();
    assert_eq!(parent.id(), store.first::<Dwarf>().unwrap().id());
}

#[test]
fn attach_already_parented_child_returns_error() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(dwarf("Thorin", 150), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();

    let d1 = store.get_by_id::<Dwarf>(0).unwrap();
    let a = store.get_by_id::<Axe>(2).unwrap();
    store.add(d1, &children![a]).unwrap();

    let d2 = store.get_by_id::<Dwarf>(1).unwrap();
    let a = store.get_by_id::<Axe>(2).unwrap();
    let err = store.add(d2, &children![a]).unwrap_err();
    assert_eq!(err, PicoError::AlreadyHasParent);
}

#[test]
fn children_returns_correct_list() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();
    store
        .add(
            Axe {
                damage: 60,
                durability: 90,
            },
            &[],
        )
        .unwrap();

    let a1 = store.get_by_id::<Axe>(1).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    store.add(d, &children![a1]).unwrap();

    let a2 = store.get_by_id::<Axe>(2).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    store.add(d, &children![a2]).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let children = store.children(&d);
    assert_eq!(children.len(), 2);
}

#[test]
fn descendants_returns_all_recursive_children() {
    let store = EntityStore::new();

    store.add(dwarf("Root", 100), &[]).unwrap();
    store.add(dwarf("Child1", 50), &[]).unwrap();
    store.add(dwarf("Child2", 50), &[]).unwrap();
    store.add(dwarf("Grandchild", 25), &[]).unwrap();

    let root = store.get_by_id::<Dwarf>(0).unwrap();
    let child1 = store.get_by_id::<Dwarf>(1).unwrap();
    store.add(root, &children![child1]).unwrap();

    let root = store.get_by_id::<Dwarf>(0).unwrap();
    let child2 = store.get_by_id::<Dwarf>(2).unwrap();
    store.add(root, &children![child2]).unwrap();

    let child2 = store.get_by_id::<Dwarf>(2).unwrap();
    let grandchild = store.get_by_id::<Dwarf>(3).unwrap();
    store.add(child2, &children![grandchild]).unwrap();

    let root = store.first::<Dwarf>().unwrap();
    let desc = store.descendants(&root);
    assert_eq!(desc.len(), 3);
}

#[test]
fn remove_removes_entity_and_descendants() {
    let store = EntityStore::new();

    store.add(dwarf("Root", 100), &[]).unwrap();
    store.add(dwarf("Child1", 50), &[]).unwrap();
    store.add(dwarf("Grandchild", 25), &[]).unwrap();

    let root = store.get_by_id::<Dwarf>(0).unwrap();
    let child1 = store.get_by_id::<Dwarf>(1).unwrap();
    store.add(root, &children![child1]).unwrap();

    let child1 = store.get_by_id::<Dwarf>(1).unwrap();
    let grandchild = store.get_by_id::<Dwarf>(2).unwrap();
    store.add(child1, &children![grandchild]).unwrap();

    let root = store.first::<Dwarf>().unwrap().entity_ref();
    store.remove(&[root]);
    assert_eq!(store.count(), 0);
}

#[test]
fn remove_single_entity_by_entity_ref() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    store.remove(&[eref]);
    assert_eq!(store.count(), 0);
}

#[test]
fn remove_batch_removes_multiple_entities() {
    let store = EntityStore::new();
    for i in 0..5 {
        store.add(dwarf(&format!("D{}", i), i), &[]).unwrap();
    }
    let e0 = store.get_by_id::<Dwarf>(0).unwrap().entity_ref();
    let e2 = store.get_by_id::<Dwarf>(2).unwrap().entity_ref();
    store.remove(&[e0, e2]);
    assert_eq!(store.count(), 3);
    assert!(store.get_by_id::<Dwarf>(0).is_none());
    assert!(store.get_by_id::<Dwarf>(2).is_none());
    assert!(store.get_by_id::<Dwarf>(1).is_some());
}

#[test]
fn remove_skips_already_dead_entities() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    store.remove(&[eref]);
    assert_eq!(store.count(), 0);
    // Removing the same EntityRef again is a no-op.
    store.remove(&[eref]);
    assert_eq!(store.count(), 0);
}

#[test]
fn clear_resets_everything() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();
    assert_eq!(store.count(), 2);
    store.clear();
    assert_eq!(store.count(), 0);
}

#[test]
fn readd_after_clear_works() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.clear();
    store.add(dwarf("Thorin", 150), &[]).unwrap();
    assert_eq!(store.count(), 1);
    let d = store.first::<Dwarf>().unwrap();
    assert_eq!(d.name, "Thorin");
}

#[test]
fn is_alive_correct_before_and_after_remove() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    assert!(store.is_alive(&d));
    let eref = d.entity_ref();
    drop(d);

    store.remove(&[eref]);
    let d = store.get_by_id::<Dwarf>(eref.id());
    assert!(d.is_none());
}

#[test]
fn resolve_returns_some_for_correct_type() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    let eref = d.entity_ref();
    drop(d);

    let resolved = store.resolve::<Dwarf>(&eref);
    assert!(resolved.is_some());
    assert_eq!(resolved.unwrap().name, "Gimli");
}

#[test]
fn resolve_returns_none_for_wrong_type() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    let eref = d.entity_ref();
    drop(d);

    assert!(store.resolve::<Axe>(&eref).is_none());
}

#[test]
fn resolve_mut_returns_some_for_correct_type() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    let eref = d.entity_ref();
    drop(d);

    {
        let mut resolved = store.resolve_mut::<Dwarf>(&eref).unwrap();
        resolved.health = 42;
    }
    let d = store.first::<Dwarf>().unwrap();
    assert_eq!(d.health, 42);
}

#[test]
fn resolve_mut_returns_none_for_wrong_type() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    let eref = d.entity_ref();
    drop(d);

    assert!(store.resolve_mut::<Axe>(&eref).is_none());
}

#[test]
fn entity_ref_id_returns_correct_value() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    let eref = d.entity_ref();
    assert_eq!(eref.id(), d.id());
}

#[test]
fn concurrent_reads() {
    use std::sync::Arc;
    use std::thread;

    let store = Arc::new(EntityStore::new());
    for i in 0..100 {
        store
            .add(dwarf(&format!("Dwarf{}", i), 100 + i), &[])
            .unwrap();
    }

    let mut handles = vec![];
    for _ in 0..4 {
        let s = store.clone();
        handles.push(thread::spawn(move || {
            let count = s.all::<Dwarf>().count();
            assert_eq!(count, 100);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn deep_hierarchy() {
    let store = EntityStore::new();

    const DEPTH: usize = 100;
    for i in 0..DEPTH {
        store.add(dwarf(&format!("Dwarf{}", i), 100), &[]).unwrap();
    }

    for i in 0..(DEPTH - 1) {
        let parent = store.get_by_id::<Dwarf>(i as u64).unwrap();
        let child = store.get_by_id::<Dwarf>((i + 1) as u64).unwrap();
        store.add(parent, &children![child]).unwrap();
    }

    let root = store.first::<Dwarf>().unwrap();
    let desc = store.descendants(&root);
    assert_eq!(desc.len(), DEPTH - 1);
}

#[test]
fn usage_example_from_plan() {
    let store = EntityStore::new();

    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();

    {
        let d1 = store.first::<Dwarf>().unwrap();
        let a1 = store.first::<Axe>().unwrap();
        assert_eq!(d1.name, "Gimli");
        assert_eq!(d1.health, 100);
        assert_eq!(a1.damage, 45);
        store.add(d1, &children![a1]).unwrap();
    }

    let d = store.first::<Dwarf>().unwrap();
    for child_ref in store.children(&d) {
        if let Some(a) = store.resolve::<Axe>(&child_ref) {
            assert_eq!(a.damage, 45);
        }
    }

    store.each::<Axe, _>(|a| {
        assert_eq!(a.durability, 80);
    });

    for d in store.all::<Dwarf>() {
        assert_eq!(d.name, "Gimli");
    }
}

#[test]
fn add_children_links_multiple() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 10,
                durability: 50,
            },
            &[],
        )
        .unwrap();
    store
        .add(
            Axe {
                damage: 20,
                durability: 60,
            },
            &[],
        )
        .unwrap();
    store
        .add(
            Axe {
                damage: 30,
                durability: 70,
            },
            &[],
        )
        .unwrap();

    let parent = store.first::<Dwarf>().unwrap();
    let c1 = store.get_by_id::<Axe>(1).unwrap();
    let c2 = store.get_by_id::<Axe>(2).unwrap();
    let c3 = store.get_by_id::<Axe>(3).unwrap();

    store.add(parent, &children![c1, c2, c3]).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let children = store.children(&d);
    assert_eq!(children.len(), 3);
}

#[test]
fn add_children_heterogeneous() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 10,
                durability: 50,
            },
            &[],
        )
        .unwrap();
    store.add(Shield { defense: 80 }, &[]).unwrap();

    let parent = store.first::<Dwarf>().unwrap();
    let axe = store.first::<Axe>().unwrap();
    let shield = store.first::<Shield>().unwrap();

    store.add(parent, &children![axe, shield]).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let children = store.children(&d);
    assert_eq!(children.len(), 2);
    let child_types: Vec<_> = children
        .iter()
        .map(|c| {
            if store.resolve::<Axe>(c).is_some() {
                "Axe"
            } else if store.resolve::<Shield>(c).is_some() {
                "Shield"
            } else {
                "Unknown"
            }
        })
        .collect();
    assert!(child_types.contains(&"Axe"));
    assert!(child_types.contains(&"Shield"));
}

#[test]
fn add_children_parent_returns_correct() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 10,
                durability: 50,
            },
            &[],
        )
        .unwrap();
    store
        .add(
            Axe {
                damage: 20,
                durability: 60,
            },
            &[],
        )
        .unwrap();

    let parent_id = store.first::<Dwarf>().unwrap().id();
    let parent = store.get_by_id::<Dwarf>(parent_id).unwrap();
    let c1 = store.get_by_id::<Axe>(1).unwrap();
    let c2 = store.get_by_id::<Axe>(2).unwrap();

    store.add(parent, &children![c1, c2]).unwrap();

    let c1 = store.get_by_id::<Axe>(1).unwrap();
    let c2 = store.get_by_id::<Axe>(2).unwrap();
    assert_eq!(store.parent(&c1).unwrap().id(), parent_id);
    assert_eq!(store.parent(&c2).unwrap().id(), parent_id);
}

#[test]
fn add_children_single_child() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let a = store.first::<Axe>().unwrap();
    store.add(d, &children![a]).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let children = store.children(&d);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id(), store.first::<Axe>().unwrap().id());
}

#[test]
fn add_children_already_parented_error() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(dwarf("Thorin", 150), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();
    store
        .add(
            Axe {
                damage: 60,
                durability: 90,
            },
            &[],
        )
        .unwrap();

    let d1 = store.get_by_id::<Dwarf>(0).unwrap();
    let a1 = store.get_by_id::<Axe>(2).unwrap();
    store.add(d1, &children![a1]).unwrap();

    let d2 = store.get_by_id::<Dwarf>(1).unwrap();
    let a1_again = store.get_by_id::<Axe>(2).unwrap();
    let a2 = store.get_by_id::<Axe>(3).unwrap();
    let err = store.add(d2, &children![a1_again, a2]).unwrap_err();
    assert_eq!(err, PicoError::AlreadyHasParent);

    let d2 = store.get_by_id::<Dwarf>(1).unwrap();
    assert_eq!(store.children(&d2).len(), 0);
}

#[test]
fn add_children_dead_entity_error() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();

    let dead_axe = store.first::<Axe>().unwrap().entity_ref();
    store.remove(&[dead_axe]);

    // Creating a new entity with a dead child fails and creates nothing.
    let err = store.add(dwarf("Thorin", 150), &[dead_axe]).unwrap_err();
    assert_eq!(err, PicoError::EntityNotAlive);
    assert_eq!(store.count(), 1);

    // Attaching a dead child to an existing entity fails too.
    let parent = store.first::<Dwarf>().unwrap();
    let err = store.add(parent, &[dead_axe]).unwrap_err();
    assert_eq!(err, PicoError::EntityNotAlive);
}

#[test]
fn add_children_all_or_nothing() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(dwarf("Thorin", 150), &[]).unwrap();
    store
        .add(
            Axe {
                damage: 45,
                durability: 80,
            },
            &[],
        )
        .unwrap();
    store
        .add(
            Axe {
                damage: 60,
                durability: 90,
            },
            &[],
        )
        .unwrap();

    let d1 = store.get_by_id::<Dwarf>(0).unwrap();
    let a2 = store.get_by_id::<Axe>(3).unwrap();
    store.add(d1, &children![a2]).unwrap();

    let d2 = store.get_by_id::<Dwarf>(1).unwrap();
    let a1 = store.get_by_id::<Axe>(2).unwrap();
    let a2_again = store.get_by_id::<Axe>(3).unwrap();
    let err = store.add(d2, &children![a1, a2_again]).unwrap_err();
    assert_eq!(err, PicoError::AlreadyHasParent);

    {
        // Read guards must be dropped before the next write-lock call.
        let d2 = store.get_by_id::<Dwarf>(1).unwrap();
        assert_eq!(store.children(&d2).len(), 0);

        let a1 = store.get_by_id::<Axe>(2).unwrap();
        assert!(store.parent(&a1).is_none());
    }

    // A failed `add` of a new entity with an unattachable child creates
    // nothing either.
    let count_before = store.count();
    let a2_again = store.get_by_id::<Axe>(3).unwrap();
    let err = store
        .add(dwarf("Newbie", 1), &children![a2_again])
        .unwrap_err();
    assert_eq!(err, PicoError::AlreadyHasParent);
    assert_eq!(store.count(), count_before);
}

#[test]
fn swap_remove_preserves_displaced_data() {
    let store = EntityStore::new();
    store.add(dwarf("A", 0x11111111), &[]).unwrap();
    store.add(dwarf("B", 0x22222222), &[]).unwrap();
    store.add(dwarf("C", 0x33333333), &[]).unwrap();

    let e0 = store.get_by_id::<Dwarf>(0).unwrap().entity_ref();
    store.remove(&[e0]);

    let b = store.get_by_id::<Dwarf>(1).unwrap();
    assert_eq!(b.name, "B");
    assert_eq!(b.health, 0x22222222);
    let c = store.get_by_id::<Dwarf>(2).unwrap();
    assert_eq!(c.name, "C");
    assert_eq!(c.health, 0x33333333);
    assert_eq!(store.count(), 2);
}

#[test]
fn swap_remove_middle_preserves_all_data() {
    let store = EntityStore::new();
    for i in 0..10 {
        store.add(dwarf(&format!("D{}", i), i * 7), &[]).unwrap();
    }

    let e3 = store.get_by_id::<Dwarf>(3).unwrap().entity_ref();
    store.remove(&[e3]);

    for i in 0..10_i32 {
        if i == 3 {
            continue;
        }
        let d = store.get_by_id::<Dwarf>(i as u64).unwrap();
        assert_eq!(d.name, format!("D{}", i));
        assert_eq!(d.health, i * 7);
    }
    assert_eq!(store.count(), 9);
}

#[test]
fn swap_remove_keeps_ids_consistent_after_removal() {
    let store = EntityStore::new();
    for i in 0..5 {
        store.add(dwarf(&format!("D{}", i), i), &[]).unwrap();
    }

    let e0 = store.get_by_id::<Dwarf>(0).unwrap().entity_ref();
    let e2 = store.get_by_id::<Dwarf>(2).unwrap().entity_ref();
    store.remove(&[e0, e2]);

    let mut ids: Vec<u64> = store.all::<Dwarf>().map(|r| r.id()).collect();
    ids.sort_unstable();
    assert_eq!(ids, vec![1, 3, 4]);

    for id in ids {
        let d = store.get_by_id::<Dwarf>(id).unwrap();
        assert_eq!(d.id(), id);
    }
    assert_eq!(store.count(), 3);
}

#[test]
fn remove_with_children_preserves_sibling_entities() {
    let store = EntityStore::new();
    for i in 0..6 {
        store.add(dwarf(&format!("D{}", i), i), &[]).unwrap();
    }
    {
        let p = store.get_by_id::<Dwarf>(0).unwrap();
        let c1 = store.get_by_id::<Dwarf>(1).unwrap();
        let c2 = store.get_by_id::<Dwarf>(2).unwrap();
        store.add(p, &children![c1, c2]).unwrap();
    }
    {
        let p = store.get_by_id::<Dwarf>(3).unwrap();
        let c1 = store.get_by_id::<Dwarf>(4).unwrap();
        let c2 = store.get_by_id::<Dwarf>(5).unwrap();
        store.add(p, &children![c1, c2]).unwrap();
    }

    let e0 = store.get_by_id::<Dwarf>(0).unwrap().entity_ref();
    store.remove(&[e0]);
    assert_eq!(store.count(), 3);

    for i in 3..6_i32 {
        let d = store.get_by_id::<Dwarf>(i as u64).unwrap();
        assert_eq!(d.name, format!("D{}", i));
        assert_eq!(d.health, i);
    }
}

#[test]
fn live_count_tracks_removals_and_clear() {
    let store = EntityStore::new();
    for i in 0..5 {
        store.add(dwarf(&format!("D{}", i), i), &[]).unwrap();
    }
    assert_eq!(store.count(), 5);
    let e1 = store.get_by_id::<Dwarf>(1).unwrap().entity_ref();
    store.remove(&[e1]);
    assert_eq!(store.count(), 4);
    let e0 = store.get_by_id::<Dwarf>(0).unwrap().entity_ref();
    store.remove(&[e0]);
    assert_eq!(store.count(), 3);
    store.clear();
    assert_eq!(store.count(), 0);
    store.add(dwarf("Fresh", 1), &[]).unwrap();
    assert_eq!(store.count(), 1);
}

#[derive(Clone)]
#[repr(align(64))]
struct OverAligned {
    tag: u32,
    payload: [u8; 40],
}

#[test]
fn over_aligned_types_stay_aligned_through_add_iterate_remove() {
    let store = EntityStore::new();
    for i in 0..8u32 {
        store
            .add(
                OverAligned {
                    tag: i,
                    payload: [i as u8; 40],
                },
                &[],
            )
            .unwrap();
    }

    for r in store.all::<OverAligned>() {
        assert_eq!(r.payload, [r.tag as u8; 40]);
    }

    let e2 = store.get_by_id::<OverAligned>(2).unwrap().entity_ref();
    store.remove(&[e2]);

    for r in store.all::<OverAligned>() {
        assert_eq!(r.payload, [r.tag as u8; 40]);
    }

    store
        .add(
            OverAligned {
                tag: 9,
                payload: [9; 40],
            },
            &[],
        )
        .unwrap();
    for r in store.all::<OverAligned>() {
        assert_eq!(r.payload, [r.tag as u8; 40]);
    }
    assert_eq!(store.count(), 8);
}

#[derive(Clone)]
struct ZeroSized;

#[test]
fn zero_sized_components_work() {
    let store = EntityStore::new();
    store.add(ZeroSized, &[]).unwrap();
    store.add(ZeroSized, &[]).unwrap();
    store.add(ZeroSized, &[]).unwrap();
    assert_eq!(store.count(), 3);

    let mut n = 0;
    store.each::<ZeroSized, _>(|_| n += 1);
    assert_eq!(n, 3);

    let e1 = store.get_by_id::<ZeroSized>(1).unwrap().entity_ref();
    store.remove(&[e1]);
    assert_eq!(store.count(), 2);
    assert_eq!(store.all::<ZeroSized>().count(), 2);

    let ids: Vec<u64> = store.all::<ZeroSized>().map(|r| r.id()).collect();
    assert_eq!(ids, vec![0, 2]);
}
