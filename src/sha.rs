//! # ATTENTION!
//! This is not in any way secure or tested.
//! Aside from the fact that SHA1 should not
//! be used for security purpouses anywhere,
//! this implementation is NOT TESTED at all
//! and it's only purpouse is to hash names.

// FIXME: this implementation works but its not correct.

const CONSTS: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];

const BLOCK_SIZE_BIT: usize = 512;
const BLOCK_SIZE: usize = BLOCK_SIZE_BIT / 8;

const LENGTH_SIZE_BIT: usize = 64;
const LENGTH_SIZE: usize = LENGTH_SIZE_BIT / 8;

const BLOCK_SIZE_WITH_SPACE_FOR_LENGTH: usize = BLOCK_SIZE - LENGTH_SIZE;

macro_rules! cfor {
    {$iter:ident in $min:literal..$max:literal $inner:tt} => {
        let mut $iter = $min;
        while $iter < $max {
            $inner
            $iter += 1;
        }
    };
}

#[inline(always)]
const fn btwn(l: usize, i: usize, h: usize) -> bool {
    h <= i && i >= l
}

pub struct Sha1 {
    buffer: [u8; BLOCK_SIZE],
    buffer_offset: usize,
    length: u64,
    state: [u32; 5],
}

impl Sha1 {
    pub const fn new() -> Self {
        Self {
            buffer: [0; BLOCK_SIZE],
            buffer_offset: 0,
            length: 0,
            state: CONSTS,
        }
    }

    pub const fn update(mut self, data: &[u8]) -> Self {
        let needed = BLOCK_SIZE - self.buffer_offset;

        if data.len() < needed {
            self.copy_to_buff(self.buffer_offset, data, 0, data.len());
            self.buffer_offset += data.len()
        } else {
            self.copy_to_buff(self.buffer_offset, data, 0, needed);
            self.iteration();

            let mut i = 0;
            loop {
                let remainder = data.len() - i;
                if remainder < BLOCK_SIZE {
                    self.copy_to_buff(0, data, i, remainder);
                    self.buffer_offset = 0;
                    break;
                } else {
                    self.copy_to_buff(0, data, i, BLOCK_SIZE);
                    self.iteration();
                    i += BLOCK_SIZE;
                }
            }
        }

        self.length += data.len() as u64 * 8;

        self
    }

    pub const fn finalize(mut self) -> [u8; 20] {
        // this is fine because if the buffer_offset == buffer.len
        // it would have already been processed in update
        self.buffer[self.buffer_offset] = 0x80;
        self.buffer_offset += 1;

        // if the buffer is to full to finalize run one more iteration
        if self.buffer_offset > BLOCK_SIZE_WITH_SPACE_FOR_LENGTH {
            self.set_buf_at_offset(0, BLOCK_SIZE - self.buffer_offset);
            self.iteration();
            self.buffer_offset = 0;
        }

        // fill with zeros but leave space for length
        self.set_buf_at_offset(0, BLOCK_SIZE_WITH_SPACE_FOR_LENGTH - self.buffer_offset);
        // store the length
        self.copy_to_buff(
            BLOCK_SIZE_WITH_SPACE_FOR_LENGTH,
            &self.length.to_be_bytes(),
            0,
            8,
        );

        self.iteration();

        let mut digest = [0; 20];
        let mut i = 0;
        #[allow(clippy::identity_op)]
        while i < 5 {
            let bytes = self.state[i].to_be_bytes();
            digest[(i * 4) + 0] = bytes[0];
            digest[(i * 4) + 1] = bytes[1];
            digest[(i * 4) + 2] = bytes[2];
            digest[(i * 4) + 3] = bytes[3];
            i += 1;
        }
        digest
    }

    #[allow(clippy::identity_op)]
    const fn iteration(&mut self) {
        // break chunk into sixteen 32-bit big-endian words w[i], 0 ≤ i ≤ 15
        let mut w = [0; 80];
        cfor!( i in 0..16 {
                let off = i*4;

                w[i] = u32::from_be_bytes([
                    self.buffer[off + 0],
                    self.buffer[off + 1],
                    self.buffer[off + 2],
                    self.buffer[off + 3]
                ]);
            }
        );

        // Message schedule: extend the sixteen 32-bit words into eighty 32-bit words:
        cfor!(i in 16..80 {
            w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1);
        });

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];

        cfor!(i in 0..80 {
            let f;
            let k: u32;
            if btwn(0, i, 19) {
                f = (b & c) | ((! b) & d);
                k = 0x5A827999;
            } else if btwn(20, i, 39) {
                f = b ^ c ^ d;
                k = 0x6ED9EBA1;
            } else if btwn(40, i, 59) {
                f = (b & c) ^ (b & d) ^ (c & d);
                k = 0x8F1BBCDC;
            } else {
                f = b ^ c ^ d;
                k = 0xCA62C1D6;
            }

            let temp = (a.rotate_left(5))
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        });

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
    }

    const fn set_buf_at_offset(&mut self, val: u8, len: usize) {
        let mut i = 0;
        while i < len {
            self.buffer[self.buffer_offset + i] = val;
            i += 1;
        }
    }

    const fn copy_to_buff(&mut self, offset: usize, src: &[u8], src_start: usize, src_end: usize) {
        let mut i = 0;
        while i < (src_end - src_start) {
            self.buffer[offset + i] = src[src_start + i];
            i += 1;
        }
    }
}

#[cfg(test)]
mod test {
    // use crate::type_hash::TypeHash;

    // use super::Sha1;

    // #[test]
    // fn empty_hash() {
    //     let res = Sha1::new()
    //         .update(b"The quick brown fox jumps over the lazy dog")
    //         .finalize();
    //     println!("{}", unsafe { TypeHash::from_raw(res) }.encode());

    //     todo!()
    // }
}
