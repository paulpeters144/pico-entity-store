use pico_entity_store::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
struct Dwarf {
    name: String,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Inventory;

#[derive(Clone)]
#[allow(dead_code)]
struct Item {
    name: String,
}

fn main() {
    let store = EntityStore::new();

    let inventory = Inventory;
    let drink = Item { name: "ale".into() };
    let food = Item { name: "apple".into() };
    let map = Item { name: "map".into() };
    let items = children![drink, food, map];
    let inv_id = store.add(inventory, &items).unwrap().id();

    let gimli = Dwarf { name: "Gimli".into() };
    let axe = Item { name: "axe".into() };
    let inv = store.get_by_id::<Inventory>(inv_id).unwrap();
    store.add(gimli, &children![axe, inv]).unwrap();

    let gimli = store.first::<Dwarf>().unwrap();

    // prints: 5 descendants
    let len = store.descendants(&gimli).len();
    println!("{} descendants", len);
}
