use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { health: 100 }, &[]).unwrap();

    if let Some(mut d) = store.first_mut::<Dwarf>() {
        d.health -= 20;
    }
    // prints: health after first_mut: 80
    println!("health after first_mut: {}", store.first::<Dwarf>().unwrap().health);
}
