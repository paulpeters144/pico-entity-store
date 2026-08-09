use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    store.add(Dwarf { name: "Thorin".into() }, &[]).unwrap();
    // prints: 2 dwarfs
    println!("{} dwarfs", store.count());
}
