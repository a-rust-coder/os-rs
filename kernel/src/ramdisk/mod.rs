use core::str;

pub struct SimpleFile<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

pub struct SimpleInitFs<'a> {
    raw: &'a [u8],
}

impl<'a> SimpleInitFs<'a> {
    pub fn new(raw: &'a [u8]) -> Self {
        Self { raw }
    }

    pub fn iter(&self) -> SimpleInitFsIter<'a> {
        SimpleInitFsIter {
            remaining: self.raw,
        }
    }
}

pub struct SimpleInitFsIter<'a> {
    remaining: &'a [u8],
}

impl<'a> Iterator for SimpleInitFsIter<'a> {
    type Item = SimpleFile<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use core::mem::size_of;

        if self.remaining.len() < size_of::<u64>() {
            return None;
        }

        let name_len = u64::from_le_bytes(self.remaining[0..8].try_into().unwrap()) as usize;
        if self.remaining.len() < 8 + name_len + 8 {
            return None;
        }

        let name_start = 8;
        let name_end = name_start + name_len;
        let name_bytes = &self.remaining[name_start..name_end];

        let data_len_offset = name_end;
        let data_len = u64::from_le_bytes(self.remaining[data_len_offset..data_len_offset + 8].try_into().unwrap()) as usize;

        let data_start = data_len_offset + 8;
        let data_end = data_start + data_len;
        if self.remaining.len() < data_end {
            return None;
        }

        let name = str::from_utf8(name_bytes).unwrap_or("<invalid>");
        let data = &self.remaining[data_start..data_end];

        self.remaining = &self.remaining[data_end..];

        Some(SimpleFile { name, data })
    }
}
