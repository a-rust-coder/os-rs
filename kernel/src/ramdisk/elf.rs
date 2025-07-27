use core::mem::size_of;
use core::ptr::copy_nonoverlapping;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Sym {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: u64,
    st_size: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Rela {
    r_offset: u64,
    r_info: u64,
    r_addend: i64,
}

// Relocation type extractor
fn r_type(info: u64) -> u32 {
    (info & 0xff) as u32
}

fn r_sym(info: u64) -> u32 {
    (info >> 32) as u32
}

unsafe fn read_struct<T: Copy>(data: &[u8], offset: usize) -> T {
    assert!(offset + size_of::<T>() <= data.len());
    unsafe { *(data.as_ptr().add(offset) as *const T) }
}

// ==== PUBLIC STRUCT ====

#[derive(Debug)]
pub struct ElfHandle<'a> {
    base_addr: usize,
    strtab: &'a [u8],
    symtab: &'static [Elf64Sym],
}

// ==== PUBLIC API ====

pub fn load_elf<'a>(data: &'a [u8], base_address: usize) -> ElfHandle<'a> {
    let ehdr: Elf64Ehdr = unsafe { read_struct(data, 0) };

    // Charge les segments PT_LOAD
    for i in 0..ehdr.e_phnum {
        let offset = ehdr.e_phoff as usize + i as usize * size_of::<Elf64Phdr>();
        let ph: Elf64Phdr = unsafe { read_struct(data, offset) };

        if ph.p_type == 1 {
            let file_start = ph.p_offset as usize;
            let file_end = file_start + ph.p_filesz as usize;
            let dest = (base_address + ph.p_vaddr as usize) as *mut u8;

            unsafe {
                copy_nonoverlapping(
                    data[file_start..file_end].as_ptr(),
                    dest,
                    ph.p_filesz as usize,
                );
                core::ptr::write_bytes(
                    dest.add(ph.p_filesz as usize),
                    0,
                    (ph.p_memsz - ph.p_filesz) as usize,
                );
            }
        }
    }

    // Trouver symtab et strtab
    let mut symtab = &[][..];
    let mut strtab = &[][..];
    for i in 0..ehdr.e_shnum {
        let off = ehdr.e_shoff as usize + i as usize * size_of::<Elf64Shdr>();
        let sh: Elf64Shdr = unsafe { read_struct(data, off) };

        const SHT_SYMTAB: u32 = 2;
        const SHT_STRTAB: u32 = 3;

        if sh.sh_type == SHT_SYMTAB {
            let count = (sh.sh_size / sh.sh_entsize) as usize;
            symtab = unsafe {
                core::slice::from_raw_parts(
                    data.as_ptr().add(sh.sh_offset as usize) as *const Elf64Sym,
                    count,
                )
            };
        } else if sh.sh_type == SHT_STRTAB && i != ehdr.e_shstrndx {
            strtab = &data[sh.sh_offset as usize..][..sh.sh_size as usize];
        }
    }

    // Applique les relocations
    for i in 0..ehdr.e_shnum {
        let off = ehdr.e_shoff as usize + i as usize * size_of::<Elf64Shdr>();
        let sh: Elf64Shdr = unsafe { read_struct(data, off) };

        const SHT_RELA: u32 = 4;
        if sh.sh_type == SHT_RELA {
            let rela_count = (sh.sh_size / sh.sh_entsize) as usize;
            for j in 0..rela_count {
                let r_off = sh.sh_offset as usize + j * size_of::<Elf64Rela>();
                let rela: Elf64Rela = unsafe { read_struct(data, r_off) };

                let reloc_addr = (base_address + rela.r_offset as usize) as *mut u64;

                match r_type(rela.r_info) {
                    8 => {
                        // R_X86_64_RELATIVE
                        unsafe {
                            *reloc_addr = base_address as u64 + rela.r_addend as u64;
                        }
                    }
                    1 => {
                        // R_X86_64_64
                        let sym_idx = r_sym(rela.r_info) as usize;
                        if sym_idx < symtab.len() {
                            let sym = symtab[sym_idx];
                            unsafe {
                                *reloc_addr =
                                    base_address as u64 + sym.st_value + rela.r_addend as u64;
                            }
                        }
                    }
                    t => panic!("Unsupported relocation type: {}", t),
                }
            }
        }
    }

    ElfHandle {
        base_addr: base_address,
        strtab,
        symtab,
    }
}

impl ElfHandle<'_> {
    pub fn get_symbol(&self, name: &str) -> usize {
        for sym in self.symtab {
            if sym.st_name as usize >= self.strtab.len() {
                continue;
            }
            let s = &self.strtab[sym.st_name as usize..];
            if let Some(len) = s.iter().position(|&c| c == 0) {
                let sym_name = core::str::from_utf8(&s[..len]).unwrap();
                if sym_name == name {
                    let addr = self.base_addr + sym.st_value as usize;
                    return addr
                }
            }
        }

        panic!("Symbol not found: {}", name);
    }
}
