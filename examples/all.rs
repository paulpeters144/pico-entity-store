use pico_entity_store::prelude::*;

#[derive(Clone)]
struct Dwarf {
    name: String,
}

fn main() {
    let store = EntityStore::new();
    store.add(Dwarf { name: "Gimli".into() }, &[]).unwrap();
    store.add(Dwarf { name: "Thorin".into() }, &[]).unwrap();
    store.add(Dwarf { name: "Balin".into() }, &[]).unwrap();

    // let mut names: Vec<String> = Vec::new();
    let names = store.all::<Dwarf>().map(|e| e.name.to_string()).collect::<String>();
    // prints: party: [Gimli, Thorin, Balin]
    println!("party: {:?}", names);
}
