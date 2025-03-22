use crate::{byte_reader::ByteReader, type_hash::TypeHash};

pub unsafe trait Storable: Sized + Clone {
    fn type_hash() -> TypeHash;
    fn encoded(&self) -> Vec<u8> {
        let mut enc = self.inner_encoded();
        let mut vec = Vec::from((enc.len() as u32).to_be_bytes());
        vec.append(&mut enc);
        vec
    }
    fn inner_encoded(&self) -> Vec<u8>;
    fn decoded(reader: ByteReader) -> Self;

    // fn inner_decoded(data: &[u8]) -> Self;
}

macro_rules! impl_all_storable_number {
    ($($ty:ident),*) => {
        $(impl_storable_number!($ty);)*
    }
}

macro_rules! impl_storable_number {
    ($ty:ident) => {
        unsafe impl Storable for $ty {
            fn type_hash() -> TypeHash {
                unsafe { TypeHash::from_str(stringify!($ty)) }
            }

            fn inner_encoded(&self) -> Vec<u8> {
                Vec::from(self.to_be_bytes())
            }

            fn decoded(reader: ByteReader) -> Self {
                Self::from_be_bytes(reader.read_byte_slice().try_into().unwrap())
            }
        }
    };
}

impl_all_storable_number!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize
);

unsafe impl Storable for String {
    fn type_hash() -> TypeHash {
        unsafe { TypeHash::from_str("String") }
    }
    fn inner_encoded(&self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }
    fn decoded(reader: ByteReader) -> Self {
        String::from_utf8(reader.read_byte_slice().to_vec()).unwrap()
    }
}

unsafe impl<T: Storable> Storable for Vec<T> {
    fn type_hash() -> TypeHash {
        unsafe { TypeHash::new("Vec", &["inner"], &[T::type_hash()]) }
    }

    fn inner_encoded(&self) -> Vec<u8> {
        self.iter().flat_map(|e| e.encoded()).collect()
    }

    fn decoded(mut reader: ByteReader) -> Self {
        let mut res = Vec::new();
        while !reader.is_at_end() {
            res.push(T::decoded(reader.reader_for_block()));
        }
        res
    }
}
