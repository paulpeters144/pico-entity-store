use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { health: 100 }, &[]).unwrap();

    let eref = store.first::<Dwarf>().unwrap().entity_ref();
    store.resolve_mut::<Dwarf>(&eref).map(|mut d| d.health -= 40);
    // prints: health after resolve_mut: 60
    println!("health after resolve_mut: {}", store.first::<Dwarf>().unwrap().health);
}
