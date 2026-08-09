use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { health: 100 }, &[]).unwrap();

    store.first_mut::<Dwarf>().map(|mut d| d.health -= 20);
    // prints: health after first_mut: 80
    println!("health after first_mut: {}", store.first::<Dwarf>().unwrap().health);
}
