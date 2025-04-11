use crate::{gen_query::ExprEntity, id::IdType, storable::Storable};

pub trait Entity: Storable {
    type Id: IdType;
    type ExprBase: ExprEntity<Self>;

    const GENERATE_ID: bool;

    fn get_id(&self) -> Self::Id;

    fn set_id(&mut self, id: Self::Id);
}
