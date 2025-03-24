use crate::sha::Sha1;

#[rustfmt::skip]
const ENCODING_TABLE: [char; 64] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f',
    'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
    'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '_'
];

const fn get_enc(d: u8) -> char {
    ENCODING_TABLE[d as usize]
}

const fn get_dec(e: u8) -> u8 {
    match e as char {
        'A'..='Z' => e - 65,
        'a'..='z' => e - 71,
        '0'..='9' => e + 4,
        '-' => 62,
        '_' => 63,
        '=' => 0,
        _ => panic!("char out of range"),
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TypeHash {
    hash: [u8; 20],
}

impl TypeHash {
    /// ## Safety:
    /// Since types must be completely unique it is unsafe to manually create them.
    pub const unsafe fn new(
        type_name: &'static str,
        field_names: &[&'static str],
        field_types: &[TypeHash],
    ) -> Self {
        let mut sha = Sha1::new().update(type_name.as_bytes());
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
        let hash = Sha1::new().update(src.as_bytes()).finalize();
        Self { hash }
    }

    pub const unsafe fn from_raw(hash: [u8; 20]) -> Self {
        Self { hash }
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.hash
    }

    pub fn encode(self) -> String {
        self.hash
            .chunks(3)
            .flat_map(|c| {
                let c1 = get_enc(c[0] >> 2);
                let c2 = get_enc((c[0] << 4 | c[1] >> 4) & 0b111111);

                if c.len() == 2 {
                    let c3 = get_enc((c[1] << 2) & 0b111111);

                    return [c1, c2, c3, '='];
                } else {
                    let c3 = get_enc((c[1] << 2 | c[2] >> 6) & 0b111111);
                    let c4 = get_enc(c[2] & 0b111111);
                    return [c1, c2, c3, c4];
                }
            })
            .collect()
    }

    pub fn decode(data: &str) -> Self {
        let hash: Vec<_> = data
            .as_bytes()
            .chunks(4)
            .flat_map(|c| {
                let enc: Vec<_> = c.into_iter().map(|i| get_dec(*i)).collect();
                if (*c.last().unwrap() as char) == '=' {
                    let c1 = (enc[0] << 2) | (enc[1] >> 4);
                    let c2 = (enc[1] << 4) | (enc[2] >> 2);

                    return vec![c1, c2];
                } else {
                    let c1 = (enc[0] << 2) | (enc[1] >> 4);
                    let c2 = (enc[1] << 4) | (enc[2] >> 2);
                    let c3 = (enc[2] << 6) | (enc[3]);
                    vec![c1, c2, c3]
                }
            })
            .collect();
        return unsafe { Self::from_raw(hash.try_into().unwrap()) };
    }
}

#[cfg(test)]
mod test {
    use super::TypeHash;

    #[test]
    fn code_roundtrip() {
        let start = unsafe { TypeHash::from_str("abc") };
        let end = TypeHash::decode(&start.clone().encode());
        assert_eq!(start, end);
    }
}
