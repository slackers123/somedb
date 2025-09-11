use std::error::Error;

use somedb::{db::Database, entity, gen_query::GenExpr, query::DbIterator};

#[entity]
#[derive(Debug, PartialEq)]
struct MyStruct {
    #[entity_id(auto_generate)]
    id: u32,
    data: String,
}

#[cfg_attr(test, test)]
fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::default()?;

    let entity = MyStruct {
        id: 0,
        data: "Hello, World!".into(),
    };

    let first_stored = db.store(entity.clone())?;
    let second_stored = db.store(entity.clone())?;
    db.store(entity.clone())?;

    db.query_mut::<MyStruct>()?
        .filter(|e| e.id().neq(second_stored.id))
        .map(|mut e| {
            e.data.push('a');
            e
        })
        .save_to_db()?;

    let first_found = db.find_by_id::<MyStruct>(first_stored.id)?;

    assert_eq!(
        first_found,
        Some(MyStruct {
            id: first_stored.id,
            data: "Hello, World!a".to_string()
        })
    );

    let second = db.find_by_id::<MyStruct>(second_stored.id)?;

    assert_eq!(second, None);

    Ok(())
}
