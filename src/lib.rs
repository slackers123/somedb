//! # SomeDb - simple database
//! A simple database for storing data.
//!
//! ## Usage
//! ```rust
//! use somedb::entity;
//! #[entity]
//! #[derive(Debug, PartialEq)]
//! struct MyStruct {
//!     #[entity_id(auto_generate)]
//!     id: i32,
//!     foo: String,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut db = somedb::db::Database::default(true)?;
//!
//!     let entity = MyStruct {
//!         id: 0, // this will be ignored because auto_generate is active
//!         foo: "bar".to_string(),
//!     };
//!
//!     let stored_entity = db.store(entity)?;
//!
//!     let loaded_entity = db.find_by_id::<MyStruct>(stored_entity.id)?.unwrap();
//!
//!     assert_eq!(stored_entity, loaded_entity);
//!
//!     Ok(())
//! }
//! ```

#[doc(hidden)]
pub mod byte_reader;
pub mod db;
pub mod entity;
pub mod entity_meta;
pub mod gen_query;
pub mod id;
pub mod query;
mod sha;
pub mod storable;
#[doc(hidden)]
pub mod type_hash;

pub use somedb_macros::Entity;
pub use somedb_macros::Storable;
pub use somedb_macros::entity;
