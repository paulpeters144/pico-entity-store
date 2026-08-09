#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::children;

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

    #[test]
    fn add_increases_count() {
        let store = EntityStore::new();
        assert_eq!(store.count(), 0);
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        assert_eq!(store.count(), 1);
        store.add(&Dwarf {
            name: "Thorin".into(),
            health: 150,
        });
        assert_eq!(store.count(), 2);
    }

    #[test]
    fn first_returns_ref_that_derefs_correctly() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        let d = store.first::<Dwarf>().unwrap();
        assert_eq!(d.name, "Gimli");
        assert_eq!(d.health, 100);
    }

    #[test]
    fn ref_id_returns_consistent_value() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        let d = store.first::<Dwarf>().unwrap();
        let id1 = d.id();
        let id2 = d.id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn refmut_deref_mut_mutates_entity_in_store() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
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
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
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
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
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
        store.add(&Axe {
            damage: 10,
            durability: 50,
        });
        store.add(&Axe {
            damage: 20,
            durability: 60,
        });
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
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
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
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        let ref_d = store.first::<Dwarf>().unwrap();
        let eref = ref_d.entity_ref();
        drop(ref_d);

        assert!(!store.update::<Axe, _>(&eref, |_| {}));
    }

    #[test]
    fn each_callback_receives_all_entities_of_type() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "A".into(),
            health: 10,
        });
        store.add(&Dwarf {
            name: "B".into(),
            health: 20,
        });
        store.add(&Axe {
            damage: 5,
            durability: 5,
        });

        let mut names = Vec::new();
        store.each::<Dwarf, _>(|d| names.push(d.name.clone()));
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"A".to_string()));
        assert!(names.contains(&"B".to_string()));
    }

    #[test]
    fn all_iterator_yields_all_entities() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "A".into(),
            health: 10,
        });
        store.add(&Dwarf {
            name: "B".into(),
            health: 20,
        });

        let dwarves: Vec<_> = store.all::<Dwarf>().collect();
        assert_eq!(dwarves.len(), 2);
        assert_eq!(dwarves[0].name, "A");
        assert_eq!(dwarves[1].name, "B");
    }

    #[test]
    fn get_by_id_returns_correct_entity() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Dwarf {
            name: "Thorin".into(),
            health: 150,
        });

        let d1 = store.first::<Dwarf>().unwrap();
        let id = d1.id();
        drop(d1);

        let d = store.get_by_id::<Dwarf>(id).unwrap();
        assert_eq!(d.name, "Gimli");
    }

    #[test]
    fn get_by_id_returns_none_for_wrong_id() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        assert!(store.get_by_id::<Dwarf>(999).is_none());
    }

    #[test]
    fn add_child_links_parent_child() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });

        let d = store.first::<Dwarf>().unwrap();
        let a = store.first::<Axe>().unwrap();
        store.add_child(d, a).unwrap();

        let d = store.first::<Dwarf>().unwrap();
        let children = store.children(&d);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id(), store.first::<Axe>().unwrap().id());
    }

    #[test]
    fn parent_returns_correct_entity_ref() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });

        let d = store.first::<Dwarf>().unwrap();
        let a = store.first::<Axe>().unwrap();
        store.add_child(d, a).unwrap();

        let a = store.first::<Axe>().unwrap();
        let parent = store.parent(&a).unwrap();
        assert_eq!(parent.id(), store.first::<Dwarf>().unwrap().id());
    }

    #[test]
    fn add_child_with_already_parented_child_returns_error() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Dwarf {
            name: "Thorin".into(),
            health: 150,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });

        let d1 = store.get_by_id::<Dwarf>(0).unwrap();
        let a = store.get_by_id::<Axe>(2).unwrap();
        store.add_child(d1, a).unwrap();

        let d2 = store.get_by_id::<Dwarf>(1).unwrap();
        let a = store.get_by_id::<Axe>(2).unwrap();
        let err = store.add_child(d2, a).unwrap_err();
        assert_eq!(err, PicoError::AlreadyHasParent);
    }

    #[test]
    fn children_returns_correct_list() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });
        store.add(&Axe {
            damage: 60,
            durability: 90,
        });

        let a1 = store.get_by_id::<Axe>(1).unwrap();
        let d = store.first::<Dwarf>().unwrap();
        store.add_child(d, a1).unwrap();

        let a2 = store.get_by_id::<Axe>(2).unwrap();
        let d = store.first::<Dwarf>().unwrap();
        store.add_child(d, a2).unwrap();

        let d = store.first::<Dwarf>().unwrap();
        let children = store.children(&d);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn descendants_returns_all_recursive_children() {
        let store = EntityStore::new();

        store.add(&Dwarf {
            name: "Root".into(),
            health: 100,
        });
        store.add(&Dwarf {
            name: "Child1".into(),
            health: 50,
        });
        store.add(&Dwarf {
            name: "Child2".into(),
            health: 50,
        });
        store.add(&Dwarf {
            name: "Grandchild".into(),
            health: 25,
        });

        let root = store.get_by_id::<Dwarf>(0).unwrap();
        let child1 = store.get_by_id::<Dwarf>(1).unwrap();
        store.add_child(root, child1).unwrap();

        let root = store.get_by_id::<Dwarf>(0).unwrap();
        let child2 = store.get_by_id::<Dwarf>(2).unwrap();
        store.add_child(root, child2).unwrap();

        let child2 = store.get_by_id::<Dwarf>(2).unwrap();
        let grandchild = store.get_by_id::<Dwarf>(3).unwrap();
        store.add_child(child2, grandchild).unwrap();

        let root = store.first::<Dwarf>().unwrap();
        let desc = store.descendants(&root);
        assert_eq!(desc.len(), 3);
    }

    #[test]
    fn remove_by_ref_removes_entity_and_descendants() {
        let store = EntityStore::new();

        store.add(&Dwarf {
            name: "Root".into(),
            health: 100,
        });
        store.add(&Dwarf {
            name: "Child1".into(),
            health: 50,
        });
        store.add(&Dwarf {
            name: "Grandchild".into(),
            health: 25,
        });

        let root = store.get_by_id::<Dwarf>(0).unwrap();
        let child1 = store.get_by_id::<Dwarf>(1).unwrap();
        store.add_child(root, child1).unwrap();

        let child1 = store.get_by_id::<Dwarf>(1).unwrap();
        let grandchild = store.get_by_id::<Dwarf>(2).unwrap();
        store.add_child(child1, grandchild).unwrap();

        let root = store.first::<Dwarf>().unwrap();
        assert!(store.remove(root));
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn remove_by_id_works_with_numeric_id() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        let id = store.first::<Dwarf>().unwrap().id();
        assert!(store.remove_by_id(id));
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn clear_resets_everything() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });
        assert_eq!(store.count(), 2);
        store.clear();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn readd_after_clear_works() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.clear();
        store.add(&Dwarf {
            name: "Thorin".into(),
            health: 150,
        });
        assert_eq!(store.count(), 1);
        let d = store.first::<Dwarf>().unwrap();
        assert_eq!(d.name, "Thorin");
    }

    #[test]
    fn is_alive_correct_before_and_after_remove() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });

        let d = store.first::<Dwarf>().unwrap();
        assert!(store.is_alive(&d));
        let id = d.id();
        drop(d);

        store.remove_by_id(id);
        let d = store.get_by_id::<Dwarf>(id);
        assert!(d.is_none());
    }

    #[test]
    fn resolve_returns_some_for_correct_type() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
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
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        let d = store.first::<Dwarf>().unwrap();
        let eref = d.entity_ref();
        drop(d);

        assert!(store.resolve::<Axe>(&eref).is_none());
    }

    #[test]
    fn resolve_mut_returns_some_for_correct_type() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
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
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        let d = store.first::<Dwarf>().unwrap();
        let eref = d.entity_ref();
        drop(d);

        assert!(store.resolve_mut::<Axe>(&eref).is_none());
    }

    #[test]
    fn entity_ref_id_returns_correct_value() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
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
            store.add(&Dwarf {
                name: format!("Dwarf{}", i),
                health: 100 + i,
            });
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
            store.add(&Dwarf {
                name: format!("Dwarf{}", i),
                health: 100,
            });
        }

        for i in 0..(DEPTH - 1) {
            let parent = store.get_by_id::<Dwarf>(i as u64).unwrap();
            let child = store.get_by_id::<Dwarf>((i + 1) as u64).unwrap();
            store.add_child(parent, child).unwrap();
        }

        let root = store.first::<Dwarf>().unwrap();
        let desc = store.descendants(&root);
        assert_eq!(desc.len(), DEPTH - 1);
    }

    #[test]
    fn usage_example_from_plan() {
        let store = EntityStore::new();

        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });

        {
            let d1 = store.first::<Dwarf>().unwrap();
            let a1 = store.first::<Axe>().unwrap();
            assert_eq!(d1.name, "Gimli");
            assert_eq!(d1.health, 100);
            assert_eq!(a1.damage, 45);
            store.add_children(d1, &children![a1]).unwrap();
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
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 10,
            durability: 50,
        });
        store.add(&Axe {
            damage: 20,
            durability: 60,
        });
        store.add(&Axe {
            damage: 30,
            durability: 70,
        });

        let parent = store.first::<Dwarf>().unwrap();
        let c1 = store.get_by_id::<Axe>(1).unwrap();
        let c2 = store.get_by_id::<Axe>(2).unwrap();
        let c3 = store.get_by_id::<Axe>(3).unwrap();

        store.add_children(parent, &children![c1, c2, c3]).unwrap();

        let d = store.first::<Dwarf>().unwrap();
        let children = store.children(&d);
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn add_children_heterogeneous() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 10,
            durability: 50,
        });
        store.add(&Shield { defense: 80 });

        let parent = store.first::<Dwarf>().unwrap();
        let axe = store.first::<Axe>().unwrap();
        let shield = store.first::<Shield>().unwrap();

        store.add_children(parent, &children![axe, shield]).unwrap();

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
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 10,
            durability: 50,
        });
        store.add(&Axe {
            damage: 20,
            durability: 60,
        });

        let parent_id = store.first::<Dwarf>().unwrap().id();
        let parent = store.get_by_id::<Dwarf>(parent_id).unwrap();
        let c1 = store.get_by_id::<Axe>(1).unwrap();
        let c2 = store.get_by_id::<Axe>(2).unwrap();

        store.add_children(parent, &children![c1, c2]).unwrap();

        let c1 = store.get_by_id::<Axe>(1).unwrap();
        let c2 = store.get_by_id::<Axe>(2).unwrap();
        assert_eq!(store.parent(&c1).unwrap().id(), parent_id);
        assert_eq!(store.parent(&c2).unwrap().id(), parent_id);
    }

    #[test]
    fn add_children_single_child() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });

        let d = store.first::<Dwarf>().unwrap();
        let a = store.first::<Axe>().unwrap();
        store.add_children(d, &children![a]).unwrap();

        let d = store.first::<Dwarf>().unwrap();
        let children = store.children(&d);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id(), store.first::<Axe>().unwrap().id());
    }

    #[test]
    fn add_children_already_parented_error() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Dwarf {
            name: "Thorin".into(),
            health: 150,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });
        store.add(&Axe {
            damage: 60,
            durability: 90,
        });

        let d1 = store.get_by_id::<Dwarf>(0).unwrap();
        let a1 = store.get_by_id::<Axe>(2).unwrap();
        store.add_child(d1, a1).unwrap();

        let d2 = store.get_by_id::<Dwarf>(1).unwrap();
        let a1_again = store.get_by_id::<Axe>(2).unwrap();
        let a2 = store.get_by_id::<Axe>(3).unwrap();
        let err = store.add_children(d2, &children![a1_again, a2]).unwrap_err();
        assert_eq!(err, PicoError::AlreadyHasParent);

        let d2 = store.get_by_id::<Dwarf>(1).unwrap();
        assert_eq!(store.children(&d2).len(), 0);
    }

    #[test]
    fn add_children_dead_entity_error() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });

        let dead_parent_id = store.first::<Dwarf>().unwrap().id();
        store.remove_by_id(dead_parent_id);
        assert_eq!(
            store.add_children_ids(dead_parent_id, &[1]),
            Err(PicoError::EntityNotAlive)
        );

        store.add(&Dwarf {
            name: "New parent".into(),
            health: 100,
        });
        let new_parent_id = store.first::<Dwarf>().unwrap().id();
        let err = store.add_children_ids(new_parent_id, &[dead_parent_id]);
        assert_eq!(err, Err(PicoError::EntityNotAlive));
    }

    #[test]
    fn add_children_all_or_nothing() {
        let store = EntityStore::new();
        store.add(&Dwarf {
            name: "Gimli".into(),
            health: 100,
        });
        store.add(&Dwarf {
            name: "Thorin".into(),
            health: 150,
        });
        store.add(&Axe {
            damage: 45,
            durability: 80,
        });
        store.add(&Axe {
            damage: 60,
            durability: 90,
        });

        let d1 = store.get_by_id::<Dwarf>(0).unwrap();
        let a2 = store.get_by_id::<Axe>(3).unwrap();
        store.add_child(d1, a2).unwrap();

        let d2 = store.get_by_id::<Dwarf>(1).unwrap();
        let a1 = store.get_by_id::<Axe>(2).unwrap();
        let a2_again = store.get_by_id::<Axe>(3).unwrap();
        let err = store.add_children(d2, &children![a1, a2_again]).unwrap_err();
        assert_eq!(err, PicoError::AlreadyHasParent);

        let d2 = store.get_by_id::<Dwarf>(1).unwrap();
        assert_eq!(store.children(&d2).len(), 0);

        let a1 = store.get_by_id::<Axe>(2).unwrap();
        assert!(store.parent(&a1).is_none());
    }
}
