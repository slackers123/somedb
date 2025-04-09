use std::error::Error;

use somedb::{db::Database, entity};

#[entity]
#[derive(Debug, PartialEq)]
struct MyStruct {
    #[entity_id]
    id: u32,
    data: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::default(true)?;

    let entity = MyStruct {
        id: 0,
        data: "Hello, World!".into(),
    };

    let stored = db.store(entity)?;

    db.delte_entity_by_id::<MyStruct>(stored.id)?;

    let loaded = db.find_by_id::<MyStruct>(stored.id)?;

    assert_eq!(loaded, None);

    Ok(())
}
