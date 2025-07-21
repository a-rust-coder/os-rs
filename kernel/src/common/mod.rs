#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtAddress(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysAddress(usize);

impl VirtAddress {
    pub const fn canonicalize_in_place(&mut self) {
        *self = Self(((self.0 << 16) as isize >> 16) as usize);
    }
    pub const fn canonicalize(self) -> Self {
        Self(((self.0 << 16) as isize >> 16) as usize)
    }
}
