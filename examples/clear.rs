use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    store.add(Dwarf { name: "Thorin".into() }, &[]).unwrap();
    store.clear();
    // prints: 0 dwarfs after clear
    println!("{} dwarfs after clear", store.count());
}
