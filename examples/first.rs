use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    // prints: first dwarf: Gimli
    println!("first dwarf: {}", store.first::<Dwarf>().unwrap().name);
}
