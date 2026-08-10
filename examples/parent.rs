use pico_entity_store::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
struct Dwarf {
    name: String,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Item {
    kind: String,
}

fn main() {
    let store = EntityStore::new();

    let axe = Item { kind: "axe".into() };
    let gimli = Dwarf { name: "Gimli".into() };
    store.add(gimli, &children![axe]).unwrap();

    let axe = store.first::<Item>().unwrap();

    // prints: parent of Battleaxe: Gimli
    let parent_id = store.parent(&axe).unwrap().id();
    let parent = store.get_by_id::<Dwarf>(parent_id).unwrap();
    println!("parent of {}: {}", axe.kind, parent.name);
}
