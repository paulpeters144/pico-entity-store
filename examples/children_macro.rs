use pico_entity_store::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
struct Axe {
    kind: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Axe { kind: "Battleaxe".into() }, &[]).unwrap();
    store.add(Axe { kind: "Dagger".into() }, &[]).unwrap();

    let a = store.first::<Axe>().unwrap();
    let b = store.first::<Axe>().unwrap();
    let refs = children![a, b];
    // prints: collected 2 EntityRefs
    println!("collected {} EntityRefs", refs.len());
}
