use pico_entity_store::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
struct Dwarf {
    name: String,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Item {
    name: String,
}

fn main() {
    let store = EntityStore::new();

    let gimili = Dwarf { name: "Gimli".into() };

    let axe = Item { name: "axe".into() };
    let drink = Item { name: "ale".into() };
    let food = Item { name: "apple".into() };

    let items = children![axe, drink, food];
    let g_ref = store.add(gimili, &items).unwrap();
    let gimili = store.get_by_id::<Dwarf>(g_ref.id()).unwrap();

    // prints: 3 children
    let count = store.children(&gimili).len();
    println!("{} children", count);
}
