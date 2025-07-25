use core::arch::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtAddress(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysAddress(usize);

impl VirtAddress {
    pub const fn canonicalize_in_place(&mut self) {
        *self = Self(((self.0 << 16) as isize >> 16) as usize);
    }
    pub const fn canonicalize(self) -> Self {
        Self(((self.0 << 16) as isize >> 16) as usize)
    }
    pub fn indices(&self) -> [usize; 4] {
        [
            ((self.0 >> 39) & 0x1FF),
            ((self.0 >> 30) & 0x1FF),
            ((self.0 >> 21) & 0x1FF),
            ((self.0 >> 12) & 0x1FF),
        ]
    }
}

pub trait Register {
    fn read(&self) -> usize;
    fn write(&self, input: usize);
}

pub struct Cr3;

impl Register for Cr3 {
    fn read(&self) -> usize {
        let value: usize;
        unsafe {
            asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        }
        value
    }

    fn write(&self, input: usize) {
        unsafe { asm!("mov cr3, {}", in(reg) input, options(nomem, nostack, preserves_flags)) };
    }
}
