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
    Dwarf { name: name.into(), health }
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
    let id0 = store.add(dwarf("Gimli", 100), &[]).unwrap().id();
    let id1 = store.add(dwarf("Thorin", 150), &[]).unwrap().id();
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
    store.add(Axe { damage: 10, durability: 50 }, &[]).unwrap();
    store.add(Axe { damage: 20, durability: 60 }, &[]).unwrap();
    store.all_mut().for_each(|a: &mut Axe| {
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

    let updated = store.update(&eref, |d: &mut Dwarf| {
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

    assert!(!store.update(&eref, |_: &mut Axe| {}));
}

#[test]
fn each_callback_receives_all_entities_of_type() {
    let store = EntityStore::new();
    store.add(dwarf("A", 10), &[]).unwrap();
    store.add(dwarf("B", 20), &[]).unwrap();
    store.add(Axe { damage: 5, durability: 5 }, &[]).unwrap();

    let mut names = Vec::new();
    store.all::<Dwarf>().for_each(|d| names.push(d.name.clone()));
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"A".to_string()));
    assert!(names.contains(&"B".to_string()));
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
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();

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
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();

    let a1 = store.first::<Axe>().unwrap();
    let parent_ref = store.add(dwarf("Parent", 100), &children![a1]).unwrap();

    let d = store.get_by_id::<Dwarf>(parent_ref.id()).unwrap();
    let children = store.children(&d);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].id(), store.first::<Axe>().unwrap().id());
}

#[test]
fn parent_returns_correct_entity_ref() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    let a = store.first::<Axe>().unwrap();
    store.add(d, &children![a]).unwrap();

    let a = store.first::<Axe>().unwrap();
    let parent = store.parent(&a).unwrap();
    assert_eq!(parent.id(), store.first::<Dwarf>().unwrap().id());
}

#[test]
fn children_returns_correct_list() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();
    store.add(Axe { damage: 60, durability: 90 }, &[]).unwrap();

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
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();
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
fn resolve_returns_none_for_wrong_type() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    let eref = d.entity_ref();
    drop(d);

    assert!(store.get_by_id::<Axe>(eref.id()).is_none());
}

#[test]
fn resolve_mut_returns_none_for_wrong_type() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    let eref = d.entity_ref();
    drop(d);

    assert!(store.get_by_id_mut::<Axe>(eref.id()).is_none());
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
        store.add(dwarf(&format!("Dwarf{}", i), 100 + i), &[]).unwrap();
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
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();

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
        if let Some(a) = store.get_by_id::<Axe>(child_ref.id()) {
            assert_eq!(a.damage, 45);
        }
    }

    store.all::<Axe>().for_each(|a| {
        assert_eq!(a.durability, 80);
    });

    for d in store.all::<Dwarf>() {
        assert_eq!(d.name, "Gimli");
    }
}

#[test]
fn add_children_heterogeneous() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(Axe { damage: 10, durability: 50 }, &[]).unwrap();
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
            if store.get_by_id::<Axe>(c.id()).is_some() {
                "Axe"
            } else if store.get_by_id::<Shield>(c.id()).is_some() {
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
fn add_children_already_parented_error() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(dwarf("Thorin", 150), &[]).unwrap();
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();
    store.add(Axe { damage: 60, durability: 90 }, &[]).unwrap();

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
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();

    let dead_axe = store.first::<Axe>().unwrap().entity_ref();
    store.remove(&[dead_axe]);

    // Creating a new entity with a dead child fails and creates nothing.
    let err = store.add(dwarf("Thorin", 150), &[ChildSource::Existing(dead_axe)]).unwrap_err();
    assert_eq!(err, PicoError::EntityNotAlive);
    assert_eq!(store.count(), 1);

    // Attaching a dead child to an existing entity fails too.
    let parent = store.first::<Dwarf>().unwrap();
    let err = store.add(parent, &[ChildSource::Existing(dead_axe)]).unwrap_err();
    assert_eq!(err, PicoError::EntityNotAlive);
}

#[test]
fn add_children_all_or_nothing() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(dwarf("Thorin", 150), &[]).unwrap();
    store.add(Axe { damage: 45, durability: 80 }, &[]).unwrap();
    store.add(Axe { damage: 60, durability: 90 }, &[]).unwrap();

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
    let err = store.add(dwarf("Newbie", 1), &children![a2_again]).unwrap_err();
    assert_eq!(err, PicoError::AlreadyHasParent);
    assert_eq!(store.count(), count_before);
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

    for &id in &[1u64, 3, 4] {
        let d = store.get_by_id::<Dwarf>(id).unwrap();
        assert_eq!(d.id(), id);
    }
    assert!(store.get_by_id::<Dwarf>(0).is_none());
    assert!(store.get_by_id::<Dwarf>(2).is_none());
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
        store.add(OverAligned { tag: i, payload: [i as u8; 40] }, &[]).unwrap();
    }

    for r in store.all::<OverAligned>() {
        assert_eq!(r.payload, [r.tag as u8; 40]);
    }

    let e2 = store.get_by_id::<OverAligned>(2).unwrap().entity_ref();
    store.remove(&[e2]);

    for r in store.all::<OverAligned>() {
        assert_eq!(r.payload, [r.tag as u8; 40]);
    }

    store.add(OverAligned { tag: 9, payload: [9; 40] }, &[]).unwrap();
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
    store.all::<ZeroSized>().for_each(|_| n += 1);
    assert_eq!(n, 3);

    let e1 = store.get_by_id::<ZeroSized>(1).unwrap().entity_ref();
    store.remove(&[e1]);
    assert_eq!(store.count(), 2);
    assert_eq!(store.all::<ZeroSized>().count(), 2);

    assert!(store.get_by_id::<ZeroSized>(0).is_some());
    assert!(store.get_by_id::<ZeroSized>(1).is_none());
    assert!(store.get_by_id::<ZeroSized>(2).is_some());
}

// ── Empty store queries ─────────────────────────────────────────────────

#[test]
fn first_returns_none_on_empty_store() {
    let store = EntityStore::new();
    assert!(store.first::<Dwarf>().is_none());
}

#[test]
fn first_mut_returns_none_on_empty_store() {
    let store = EntityStore::new();
    assert!(store.first_mut::<Dwarf>().is_none());
}

#[test]
fn all_yields_zero_items_on_empty_store() {
    let store = EntityStore::new();
    assert_eq!(store.all::<Dwarf>().count(), 0);
}

#[test]
fn all_mut_yields_zero_items_on_empty_store() {
    let store = EntityStore::new();
    assert_eq!(store.all_mut::<Dwarf>().count(), 0);
}

// ── update error cases ──────────────────────────────────────────────────

#[test]
fn update_returns_false_for_dead_entity() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    store.remove(&[eref]);
    assert!(!store.update::<Dwarf, _>(&eref, |d| d.health -= 10));
}

// ── Hierarchy “none” cases ──────────────────────────────────────────────

#[test]
fn parent_returns_none_for_root_entity() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    assert!(store.parent(&d).is_none());
}

#[test]
fn children_returns_empty_for_leaf_entity() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    assert!(store.children(&d).is_empty());
}

#[test]
fn descendants_returns_empty_for_leaf_entity() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first::<Dwarf>().unwrap();
    assert!(store.descendants(&d).is_empty());
}

// ── RefMut guard API ────────────────────────────────────────────────────

#[test]
fn refmut_id_returns_correct_value() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first_mut::<Dwarf>().unwrap();
    assert_eq!(d.id(), 0);
}

#[test]
fn refmut_entity_ref_returns_correct_value() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first_mut::<Dwarf>().unwrap();
    let eref = d.entity_ref();
    assert_eq!(eref.id(), 0);
}

#[test]
fn refmut_immutable_deref_reads_correctly() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let d = store.first_mut::<Dwarf>().unwrap();
    assert_eq!(d.name, "Gimli");
    assert_eq!(d.health, 100);
}

// ── EntityRef trait impls ───────────────────────────────────────────────

#[test]
fn entity_ref_clone_produces_equal_copy() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    let cloned = eref;
    assert_eq!(cloned.id(), eref.id());
}

#[test]
fn entity_ref_copy_allows_reuse_after_pass_by_value() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    let copy = eref;
    assert_eq!(copy.id(), 0);
    assert_eq!(eref.id(), 0);
}

#[test]
fn entity_ref_eq_and_hash_work_in_hashset() {
    use std::collections::HashSet;
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    store.add(dwarf("Thorin", 150), &[]).unwrap();
    let e0 = store.get_by_id::<Dwarf>(0).unwrap().entity_ref();
    let e1 = store.get_by_id::<Dwarf>(1).unwrap().entity_ref();
    let mut set = HashSet::new();
    set.insert(e0);
    set.insert(e1);
    set.insert(e0);
    assert_eq!(set.len(), 2);
}

#[test]
fn entity_ref_debug_contains_id() {
    let store = EntityStore::new();
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    let debug = format!("{:?}", eref);
    assert!(debug.contains("0"), "debug output should contain entity id: {debug}");
}

// ── PicoError Display ───────────────────────────────────────────────────

#[test]
fn pico_error_display_formats_correctly() {
    assert_eq!(PicoError::TypeNotRegistered.to_string(), "type not registered");
    assert_eq!(PicoError::AlreadyHasParent.to_string(), "entity already has a parent");
    assert_eq!(PicoError::EntityNotAlive.to_string(), "entity is not alive");
}

// ── Iterator size_hint / count ──────────────────────────────────────────

#[test]
fn all_iter_size_hint_and_count_match() {
    let store = EntityStore::new();
    for i in 0..5 {
        store.add(dwarf(&format!("D{}", i), i), &[]).unwrap();
    }
    let iter = store.all::<Dwarf>();
    assert_eq!(iter.size_hint(), (5, Some(5)));
    assert_eq!(iter.count(), 5);
}

#[test]
fn all_mut_iter_size_hint_and_count_match() {
    let store = EntityStore::new();
    for i in 0..5 {
        store.add(dwarf(&format!("D{}", i), i), &[]).unwrap();
    }
    let iter = store.all_mut::<Dwarf>();
    assert_eq!(iter.size_hint(), (5, Some(5)));
    assert_eq!(iter.count(), 5);
}

// ── Default trait ───────────────────────────────────────────────────────

#[test]
fn default_creates_empty_store() {
    let store = EntityStore::default();
    assert_eq!(store.count(), 0);
    store.add(dwarf("Gimli", 100), &[]).unwrap();
    assert_eq!(store.count(), 1);
}

// ── Swap-remove interaction with read queries ───────────────────────────

#[test]
fn first_returns_swapped_entity_after_removing_first() {
    let store = EntityStore::new();
    store.add(dwarf("D0", 0), &[]).unwrap();
    store.add(dwarf("D1", 1), &[]).unwrap();
    store.add(dwarf("D2", 2), &[]).unwrap();

    let e0 = store.get_by_id::<Dwarf>(0).unwrap().entity_ref();
    store.remove(&[e0]);

    let d = store.first::<Dwarf>().unwrap();
    assert_eq!(d.name, "D2");
    assert_eq!(d.health, 2);
}

#[test]
fn all_iter_reflects_state_after_removal() {
    let store = EntityStore::new();
    for i in 0..5 {
        store.add(dwarf(&format!("D{}", i), i), &[]).unwrap();
    }
    let e1 = store.get_by_id::<Dwarf>(1).unwrap().entity_ref();
    let e3 = store.get_by_id::<Dwarf>(3).unwrap().entity_ref();
    store.remove(&[e1, e3]);

    assert_eq!(store.count(), 3);
    assert_eq!(store.all::<Dwarf>().count(), 3);

    for &id in &[0u64, 2, 4] {
        let d = store.get_by_id::<Dwarf>(id).unwrap();
        assert_eq!(d.name, format!("D{id}"));
        assert_eq!(d.health, id as i32);
    }
    assert!(store.get_by_id::<Dwarf>(1).is_none());
    assert!(store.get_by_id::<Dwarf>(3).is_none());
}

