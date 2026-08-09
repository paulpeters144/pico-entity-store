use pico_entity_store::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    store.remove(&[eref]);
    // prints: 0 dwarfs after remove
    println!("{} dwarfs after remove", store.count());
}
