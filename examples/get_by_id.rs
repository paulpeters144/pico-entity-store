use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    // prints: id 0 = Gimli
    println!("id 0 = {}", store.get_by_id::<Dwarf>(0).unwrap().name);
}
