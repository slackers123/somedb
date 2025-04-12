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

### Queries
There are two types of queries. Simple quereis are simply iterators over a list of entities.
```rust
let alans = db.query::<Person>()?
    .filter(|p| p.first_name == "Alan")
    .collect::<Vec<_>>()
```
Complex quereies are inspired by polars queries and will allow for more complex optimizations
in the future and they can be saved back to the database immediately.
```rust
db.query_mut::<Person>()?
    .filter(|p| p.first_name().eq("Alan"))
    .save_to_db()?;
```

## Features
- [x] Store entities
- [x] Load all entities
- [x] Load entities by id
- [x] Delete entities by id
- [x] general query iterator
- [x] better queries to support future storage model

## Future Improvements
- [ ] improved storage model to avoid loading entire database into memory
