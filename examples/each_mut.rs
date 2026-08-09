use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { health: 100 }, &[]).unwrap();
    store.add(Dwarf { health: 80 }, &[]).unwrap();

    store.each_mut::<Dwarf, _>(|d| d.health += 10);
    store.each::<Dwarf, _>(|d| {
        // prints:
        //   each_mut hp: 110
        //   each_mut hp: 90
        println!("each_mut hp: {}", d.health);
    });
}
