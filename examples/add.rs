use pico_entity_store::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    let d_ref = store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    // prints: Gimli id = 0
    println!("Gimli id = {}", d_ref.id());
}
