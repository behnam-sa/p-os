use conquer_once::spin::OnceCell;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

static IDT: OnceCell<InterruptDescriptorTable> = OnceCell::uninit();

pub fn init_idt() {
    let idt = IDT.get_or_init(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    });
    idt.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    log::warn!("Exception: Breakpoint\n{stack_frame:#?}");
}
