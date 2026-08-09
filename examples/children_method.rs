use pico_entity_store::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
struct Dwarf {
    name: String,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Axe {
    kind: String,
}

fn main() {
    let store = EntityStore::new();
    let axe_id = store.add(Axe { kind: "Axe".into() }, &[]).unwrap().id();
    let dag_id = store.add(Axe { kind: "Dagger".into() }, &[]).unwrap().id();

    let a = store.get_by_id::<Axe>(axe_id).unwrap();
    let s = store.get_by_id::<Axe>(dag_id).unwrap();
    store.add(Dwarf { name: "Gimli".into() }, &children![a, s]).unwrap();

    // prints: 2 children
    let count = store.children(&store.first::<Dwarf>().unwrap()).len();
    println!("{} children", count);
}
