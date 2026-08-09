use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { health: 100 }, &[]).unwrap();

    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    store.update(&eref, |d: &mut Dwarf| d.health -= 10);
    // prints: update health: 90
    println!("update health: {}", store.first::<Dwarf>().unwrap().health);
}
