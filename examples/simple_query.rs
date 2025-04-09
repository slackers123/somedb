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

    let mut entity = MyStruct {
        id: 0,
        data: "Hello, World!".into(),
    };

    let first = db.store(entity.clone())?;
    entity.id = 1;
    db.store(entity.clone())?;
    entity.id = 2;
    let third = db.store(entity.clone())?;

    let res: Vec<_> = db.query::<MyStruct>()?.filter(|e| e.id != 1).collect();

    assert_eq!(res, vec![first, third]);

    Ok(())
}
