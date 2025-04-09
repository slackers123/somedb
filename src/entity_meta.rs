use crate::{
    db::{DbError, DbResult},
    entity::Entity,
    storable::Storable,
    type_hash::TypeHash,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct EntityMeta<T: Entity> {
    pub last_id: T::Id,
    pub entities: Vec<T>,
}
unsafe impl<T: Entity> Storable for EntityMeta<T> {
    fn type_hash() -> crate::type_hash::TypeHash {
        unsafe { TypeHash::new("", &[], &[]) }
    }

    fn inner_encoded(&self) -> Vec<u8> {
        let mut res = String::from(VERSION).encoded();
        res.append(&mut self.last_id.encoded());
        res.append(&mut self.entities.encoded());

        res
    }

    fn decoded(mut reader: crate::byte_reader::ByteReader) -> DbResult<Self> {
        if String::decoded(reader.reader_for_block())? != VERSION {
            return Err(DbError::InvalidFileVersion);
        }
        Ok(Self {
            last_id: T::Id::decoded(reader.reader_for_block())?,
            entities: Vec::decoded(reader.reader_for_block())?,
        })
    }
}
