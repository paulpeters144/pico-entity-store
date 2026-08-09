use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { health: 100 }, &[]).unwrap();

    if let Some(mut d) = store.get_by_id_mut::<Dwarf>(0) {
        d.health -= 30;
    }
    // prints: health after get_by_id_mut: 70
    println!("health after get_by_id_mut: {}", store.get_by_id::<Dwarf>(0).unwrap().health);
}
