# SomeDb

Extremely simple database to store data.

## Usage
```rust
use somedb::entity;
#[entity]
#[derive(Debug)]
struct MyStruct {
    #[entity_id(auto_generate)]
    id: i32,
    foo: String,
}

fn main() {
    let mut db = somedb::db::Database::default(true).unwrap();
    let entity = MyStruct {
        id: 0, // this will be ignored because auto_generate is active
        foo: "bar".to_string(),
    };

    db.store(entity).unwrap();

    let saved_entity = db.find_by_id::<MyStruct>(1).unwrap();
    println!("{saved_entity:?}");
}
```
