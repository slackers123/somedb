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

    let first = db.store(entity.clone())?;
    let second = db.store(entity.clone())?;
    let third = db.store(entity.clone())?;

    let loaded = db.read_all::<MyStruct>()?;

    assert_eq!(loaded, vec![first, second, third]);

    Ok(())
}
