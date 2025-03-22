use std::{any::type_name, error::Error};

use somedb::{Storable, db::Database, entity::Entity};

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::default()?;

    let _ = db.delete_entity_store::<MyStruct>();

    let some_data = MyStruct {
        id: 1,
        data: String::from("value"),
    };

    let mut some_data = db.store(some_data)?;

    some_data.data.push_str(" of everything.");

    db.update_entity(some_data)?;

    let rows = db.find_by_id::<MyStruct>(1)?;
    println!("{rows:?}");

    Ok(())
}

#[derive(Storable, Debug, Clone)]
struct MyStruct {
    id: u32,
    data: String,
}

impl Entity for MyStruct {
    type Id = u32;
    fn get_id(&self) -> u32 {
        return self.id;
    }
}
