use crate::storable::Storable;

/// An Id used for indexing in SomeDb
///
/// ## Note
/// It is recommended to use one of the basic
/// int types since they are guaranteed to be
/// supported in future releases.
pub trait IdType: Storable + PartialEq + PartialOrd + Copy {
    /// function used to generate the next id.
    ///
    /// ## Note
    /// Using the last id is completely optional, for exmple
    /// a uuid may be completely randomly generated and
    /// not depend on the last value at all.
    fn generate(last_id: Self) -> Self;

    /// Initial id used for the first entry in the
    /// database.
    fn initial() -> Self;
}

macro_rules! gen_number_id {
    ($ty:ident) => {
        impl IdType for $ty {
            fn generate(last_id: Self) -> Self {
                last_id + 1
            }

            fn initial() -> Self {
                0
            }
        }
    };
}

macro_rules! gen_multiple_number_id {
    ($($ty:ident),*) => {
        $(gen_number_id!($ty);)*
    };
}

gen_multiple_number_id!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
