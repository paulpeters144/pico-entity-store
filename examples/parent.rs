use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

#[derive(Clone)]
struct Axe {
    kind: String,
}

fn main() {
    let store = EntityStore::new();
    let a = store.get_by_id::<Axe>(
        store.add(Axe { kind: "Battleaxe".into() }, &[]).unwrap()
    ).unwrap();
    store.add(Dwarf { name: "Gimli".into() }, &children![a]).unwrap();

    let axe = store.first::<Axe>().unwrap();
    // prints: parent of Battleaxe: Gimli
    println!("parent of {}: {}", axe.kind,
        store.resolve::<Dwarf>(&store.parent(&axe).unwrap()).unwrap().name);
}
