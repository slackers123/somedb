use std::error::Error;

use somedb::{db::Database, entity, query::DbIterator};

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

    db.store(entity.clone())?;
    entity.id = 1;
    db.store(entity.clone())?;
    entity.id = 2;
    db.store(entity.clone())?;

    db.query_mut::<MyStruct>()?
        .filter(|e| e.id != 1)
        .map(|mut e| {
            e.data.push('a');
            e
        })
        .save_to_db()?;

    let first = db.find_by_id::<MyStruct>(0)?;

    assert_eq!(
        first,
        Some(MyStruct {
            id: 0,
            data: "Hello, World!a".into()
        })
    );

    let second = db.find_by_id::<MyStruct>(1)?;

    assert_eq!(second, None);

    Ok(())
}
