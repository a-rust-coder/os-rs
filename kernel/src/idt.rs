use core::arch::asm;

#[repr(C)]
#[derive(Debug)]
pub struct InterruptStackFrame {
    pub instruction_pointer: usize,
    pub code_segment: usize,
    pub cpu_flags: usize,
    pub stack_pointer: usize,
    pub stack_segment: usize,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_middle: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_middle: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    fn new(handler: usize) -> Self {
        Self {
            offset_low: handler as u16,
            selector: 0x08,                // code segment (typiquement GDT[1])
            options: 0b1000_1110_00000000, // présent, DPL=0, interrupt gate (0x8E00)
            offset_middle: (handler >> 16) as u16,
            offset_high: (handler >> 32) as u32,
            zero: 0,
        }
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: usize,
}

// IDT statique en mémoire
static mut IDT: [IdtEntry; 256] = [IdtEntry::missing(); 256];

extern "x86-interrupt" fn handler_pf(_stack: &mut InterruptStackFrame, _error_code: usize) {
    // serial_println!("EXCEPTION: Page Fault");
    // serial_println!("{}\n{:#?}", error_code, stack);
    // serial_println!("Page: {}", read_cr2());

    loop {}
}

extern "x86-interrupt" fn handler_gp(_stack: &mut InterruptStackFrame, _error_code: usize) {
    // serial_println!("EXCEPTION: General Protection Fault");
    loop {}
}

extern "x86-interrupt" fn handler_df(_stack: &mut InterruptStackFrame, _error_code: usize) {
    // serial_println!("EXCEPTION: Double Fault");
    loop {}
}

extern "x86-interrupt" fn handler_ud(_stack: &mut InterruptStackFrame) {
    // serial_println!("EXCEPTION: Undefined Instruction");
    loop {}
}

extern "x86-interrupt" fn handler_default(_stack: &mut InterruptStackFrame) {
    // serial_println!("EXCEPTION: Interrupt");
    loop {}
}

/// Initialise et charge l'IDT
pub fn init_idt() {
    unsafe {
        IDT[6] = IdtEntry::new(handler_ud as usize); // #UD
        IDT[8] = IdtEntry::new(handler_df as usize); // #DF
        IDT[13] = IdtEntry::new(handler_gp as usize); // #GP
        IDT[14] = IdtEntry::new(handler_pf as usize); // #PF

        for i in 0..256 {
            if IDT[i].offset_low == 0 && IDT[i].offset_middle == 0 && IDT[i].offset_high == 0 {
                IDT[i] = IdtEntry::new(handler_default as usize);
            }
        }

        let idt_ptr = IdtPointer {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: &raw const IDT as usize,
        };

        asm!("lidt [{}]", in(reg) &idt_ptr, options(readonly, nostack));
    }

    // serial_println!("IDT installée !");
}
