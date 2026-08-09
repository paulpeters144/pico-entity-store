use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { health: 100 }, &[]).unwrap();

    let mut d = store.first_mut::<Dwarf>().unwrap();
    d.health -= 25;
    // prints: RefMut<Dwarf>: id=0, hp=75
    println!("RefMut<Dwarf>: id={}, hp={}", d.id(), d.health);
}
