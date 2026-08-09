use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    health: i32,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { health: 100 }, &[]).unwrap();
    store.add(Dwarf { health: 80 }, &[]).unwrap();

    store.all_mut::<Dwarf>().for_each(|d| d.health += 10);
    store.all::<Dwarf>().for_each(|d| {
        // prints:
        //   all_mut hp: 110
        //   all_mut hp: 90
        println!("all_mut hp: {}", d.health);
    });
}
