use std::error::Error;

use somedb::{db::Database, entity};

#[derive(Debug, PartialEq)]
#[entity]
struct MyStruct {
    #[entity_id(auto_generate)]
    id: u32,
    data: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::default(true)?;

    let entity = MyStruct {
        id: 0, // this value does not matter
        data: "Hello, World!".into(),
    };

    let first = db.store(entity.clone())?;
    let second = db.store(entity.clone())?;

    assert_ne!(first.id, second.id);

    Ok(())
}
