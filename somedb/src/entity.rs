use crate::{id::IdType, storable::Storable};

pub trait Entity: Storable {
    type Id: IdType;
    const GENERATE_ID: bool;

    fn get_id(&self) -> Self::Id;

    fn set_id(&mut self, id: Self::Id);
}
