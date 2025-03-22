use sha2_const::Sha256;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TypeHash {
    hash: [u8; 32],
}

impl TypeHash {
    /// ## Safety:
    /// Since types must be completely unique it is unsafe to manually create them.
    pub const unsafe fn new(
        type_name: &'static str,
        field_names: &[&'static str],
        field_types: &[TypeHash],
    ) -> Self {
        let mut sha = Sha256::new().update(type_name.as_bytes());
        let mut i = 0;
        while i < field_names.len() {
            sha = sha.update(field_names[i].as_bytes());
            sha = sha.update(field_types[i].as_bytes());
            i += 1;
        }
        Self {
            hash: sha.finalize(),
        }
    }

    /// ## Safety:
    /// Since types must be completely unique it is unsafe to manually create them.
    pub const unsafe fn from_str(src: &'static str) -> Self {
        let hash = Sha256::new().update(src.as_bytes()).finalize();
        Self { hash }
    }

    pub const unsafe fn from_raw(hash: [u8; 32]) -> Self {
        Self { hash }
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.hash
    }
}
