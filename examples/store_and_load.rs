use std::error::Error;

use somedb::{db::Database, entity};

#[entity]
#[derive(Debug, PartialEq)]
struct MyStruct {
    #[entity_id(auto_generate)]
    id: u32,
    data: String,
}

#[cfg_attr(test, test)]
fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::default(true)?;

    let entity = MyStruct {
        id: 0,
        data: "Hello, World!".into(),
    };

    let stored = db.store(entity)?;

    let loaded = db.find_by_id(stored.id)?;

    assert_eq!(Some(stored), loaded);

    Ok(())
}
