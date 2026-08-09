use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into(), health: 100 }, &[]).unwrap();

    let d = store.first::<Dwarf>().unwrap();
    // prints: Ref<Dwarf>: Gimli (id=0, hp=100)
    println!("Ref<Dwarf>: {} (id={}, hp={})", d.name, d.id(), d.health);
}
