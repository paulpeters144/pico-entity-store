use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

#[derive(Clone)]
struct Axe {
    kind: String,
}

fn main() {
    let store = EntityStore::new();
    let a = store.get_by_id::<Axe>(store.add(Axe { kind: "Axe".into() }, &[]).unwrap()).unwrap();
    store.add(Dwarf { name: "Gimli".into() }, &children![a]).unwrap();
    // prints: 1 descendant
    println!("{} descendant", store.descendants(&store.first::<Dwarf>().unwrap()).len());
}
