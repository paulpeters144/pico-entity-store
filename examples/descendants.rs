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

    let dwarf = Dwarf { name: "Gimli".into() };
    let axe = Axe { kind: "Axe".into() };

    store.add(dwarf, &children![axe]).unwrap();

    let guard = store.first::<Dwarf>().unwrap();

    // prints: 1 descendant
    let len = store.descendants(&guard).len();
    println!("{} descendant", len);
}
