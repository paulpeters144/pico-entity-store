use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    store.add(Dwarf { name: "Thorin".into() }, &[]).unwrap();
    // prints: collected 2 dwarfs
    println!("collected {} dwarfs", store.all::<Dwarf>().count());
}
