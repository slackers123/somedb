//! # SomeDb - simple database
//! A simple database for storing data.
//!
//! ## Usage
//! ```rust
//! use somedb::entity;
//! #[entity]
//! #[derive(Debug)]
//! struct MyStruct {
//!     #[entity_id(auto_generate)]
//!     id: i32,
//!     foo: String,
//! }
//!
//! fn main() {
//!     let mut db = somedb::db::Database::default(true).unwrap();
//!     let entity = MyStruct {
//!         id: 0, // this will be ignored because auto_generate is active
//!         foo: "bar".to_string(),
//!     };
//!
//!     db.store(entity).unwrap();
//!
//!     let saved_entity = db.find_by_id::<MyStruct>(1).unwrap();
//!     println!("{saved_entity:?}");
//! }
//! ```

#[doc(hidden)]
pub mod byte_reader;
pub mod db;
pub mod entity;
pub mod entity_meta;
pub mod id;
pub mod storable;
#[doc(hidden)]
pub mod type_hash;

pub use somedb_macros::Entity;
pub use somedb_macros::Storable;
pub use somedb_macros::entity;
