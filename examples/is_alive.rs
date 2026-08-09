use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();

    let alive = store.is_alive(&store.first::<Dwarf>().unwrap());
    // prints: alive = true
    println!("alive = {}", alive);
}
