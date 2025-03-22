use crate::storable::Storable;

pub trait Entity: Storable {
    type Id: PartialEq + PartialOrd;
    fn get_id(&self) -> Self::Id;
}
