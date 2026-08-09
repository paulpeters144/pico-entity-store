use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    let id = store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    // prints: Gimli id = 0
    println!("Gimli id = {}", id);
}
