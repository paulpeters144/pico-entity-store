use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    // prints: resolved: Gimli
    println!("resolved: {}", store.resolve::<Dwarf>(&eref).unwrap().name);
}
