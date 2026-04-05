pub struct ArchiveCursor<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> ArchiveCursor<'a> {
    pub fn read_u32(&mut self) -> u32 {
        let x = u32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        x
    }

    pub fn read_u16(&mut self) -> u16 {
        let x = u16::from_le_bytes(self.data[self.pos..self.pos + 2].try_into().unwrap());
        self.pos += 2;
        x
    }

    pub fn read_u8(&mut self) -> u8 {
        let x = self.data[self.pos];
        self.pos += 1;
        x
    }

    pub fn read_f32(&mut self) -> f32 {
        let x = f32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        x
    }

    pub fn read_string(&mut self, len: usize) -> String {
        let x = String::from_utf8_lossy(&self.data[self.pos..self.pos + len]);
        self.pos += len;
        x.to_string()
    }

    pub fn read_slice(&mut self, len: usize) -> &[u8] {
        let x = &self.data[self.pos..self.pos + len];
        self.pos += len;
        x
    }
}
