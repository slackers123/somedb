use std::error::Error;

use somedb::{
    db::{Database, DbError},
    entity,
};

#[derive(Debug)]
#[entity]
struct Foo {
    #[entity_id(auto_generate)]
    id: u32,
}

#[test]
fn load_without_create() -> Result<(), Box<dyn Error>> {
    let db = Database::default(true)?;
    assert_eq!(
        db.find_by_id::<Foo>(0)
            .expect_err("find should not succed in this case"),
        DbError::TypeNotFound
    );

    Ok(())
}

#[test]
fn multiple_db() -> Result<(), Box<dyn Error>> {
    let db = Database::default(true)?;

    assert_eq!(
        Database::default(true).expect_err("creating two databases should not be possible"),
        DbError::DbInstanceExists
    );

    drop(db);

    Ok(())
}
