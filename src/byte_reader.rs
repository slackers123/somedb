#[derive(Debug, Clone)]
pub struct ByteReader<'a> {
    src: &'a [u8],
    start: usize,
    end: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(src: &'a [u8]) -> ByteReader<'a> {
        Self {
            src,
            start: 0,
            end: src.len(),
        }
    }

    pub fn reader_for_block(&mut self) -> ByteReader<'a> {
        if self.is_at_end() {
            return self.clone();
        }
        let len = self.read_len();
        self.start += len as usize;
        ByteReader {
            src: self.src,
            start: self.start - len as usize,
            end: self.start,
        }
    }

    fn read_len(&mut self) -> u32 {
        self.start += 4;
        u32::from_be_bytes(self.src[(self.start - 4)..self.start].try_into().unwrap())
    }

    pub fn read_byte_slice(&'a self) -> &'a [u8] {
        &self.src[self.start..self.end]
    }

    pub fn is_at_end(&self) -> bool {
        self.start == self.end
    }
}
