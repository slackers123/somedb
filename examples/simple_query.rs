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
    let mut db = Database::default()?;

    let entity = MyStruct {
        id: 0,
        data: "Hello, World!".into(),
    };

    let _first = db.store(entity.clone())?;
    let second = db.store(entity.clone())?;
    let _third = db.store(entity.clone())?;

    let res: Vec<_> = db
        .query::<MyStruct>()?
        // now we filter out the second entry
        .filter(|e| e.id != second.id)
        .collect();

    // there is now no longer an entry with the second id
    assert_eq!(res.iter().find(|s| s.id == second.id), None);

    Ok(())
}
