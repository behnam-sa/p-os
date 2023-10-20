use crate::{gdt, print};
use conquer_once::spin::Lazy;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

pub(crate) const PIC_1_OFFSET: u8 = 32;
pub(crate) const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub(crate) static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(create_idt);

macro_rules! add_hardware_interrupt {
    ($idt:expr, $interrupt_id:expr, $callback:ident) => {{
        extern "x86-interrupt" fn interrupt_handler(_stack_frame: InterruptStackFrame) {
            $callback();

            unsafe {
                PICS.lock().notify_end_of_interrupt($interrupt_id as u8);
            }
        }

        $idt[$interrupt_id as usize].set_handler_fn(interrupt_handler);
    }};
}

fn create_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX)
    };
    add_hardware_interrupt!(idt, InterruptIndex::Timer, timer_interrupt_handler);
    add_hardware_interrupt!(idt, InterruptIndex::Keyboard, keyboard_interrupt_handler);

    idt
}

pub(crate) fn init() {
    IDT.load();
    unsafe {
        let mut pics = PICS.lock();
        pics.initialize();
        pics.write_masks(0b1111_1100, 0b1111_1111)
    };

    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::warn!("Exception: Breakpoint\n{stack_frame:#?}");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    let cr2_value = Cr2::read();

    panic!(
        "Exception: PAGE FAULT\n\
        Accessed Address: {cr2_value:?}\n\
        Error Code: {error_code:?}\n\
        {stack_frame:#?}"
    );
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Exception: Double fault\n{stack_frame:#?}");
}

fn timer_interrupt_handler() {
    print!(".");
}

fn keyboard_interrupt_handler() {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    static KEYBOARD: Lazy<Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>>> = Lazy::new(|| {
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        ))
    });

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                _ => (),
            }
        }
    }
}
