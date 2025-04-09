# SomeDb

Extremely simple database to store data.

## Usage
```rust
use somedb::entity;
#[entity]
#[derive(Debug, PartialEq)]
struct MyStruct {
    #[entity_id(auto_generate)]
    id: i32,
    foo: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = somedb::db::Database::default(true)?;

    let entity = MyStruct {
        id: 0, // this will be ignored because auto_generate is active
        foo: "bar".to_string(),
    };

    let stored_entity = db.store(entity)?;

    let loaded_entity = db.find_by_id::<MyStruct>(stored_entity.id)?.unwrap();

    assert_eq!(stored_entity, loaded_entity);

    Ok(())
}
```

## Features
- [x] Store entities
- [x] Load all entities
- [x] Load entities by id
- [x] Delete entities by id
- [ ] general query iterator

## Future Improvements
- [ ] improved storage model to avoid loading entire database into memory
