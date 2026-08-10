use pico_entity_store::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
struct Dwarf {
    name: String,
}

#[derive(Clone)]
#[allow(dead_code)]
struct Axe {
    kind: String,
}

fn main() {
    let store = EntityStore::new();

    let axe = Axe { kind: "Battleaxe".into() };
    let axe_ref = store.add(axe, &[]).unwrap();

    let gimli = Dwarf { name: "Gimli".into() };
    let axe = store.get_by_id::<Axe>(axe_ref.id()).unwrap();
    store.add(gimli, &children![axe]).unwrap();

    let axe = store.first::<Axe>().unwrap();

    // prints: parent of Battleaxe: Gimli
    let parent = store.get_by_id::<Dwarf>(store.parent(&axe).unwrap().id()).unwrap();
    println!("parent of {}: {}", axe.kind, parent.name);
}
