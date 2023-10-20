use core::fmt;

type PrintHandler = &'static (dyn Fn(fmt::Arguments) + Sync);

use conquer_once::{spin::OnceCell, TryInitError};

static PRINT_HANDLER: OnceCell<PrintHandler> = OnceCell::uninit();

pub fn set_print_handler(handler: PrintHandler) -> Result<(), TryInitError> {
    PRINT_HANDLER.try_init_once(|| handler)
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    if let Some(print_handler) = PRINT_HANDLER.get() {
        print_handler(args);
    }
}

/// Prints to the terminal output
// #[macro_export]
// #[macro_pub]
pub macro print {
    ($($arg:tt)*) => {
        $crate::io::_print(format_args!($($arg)*))
    }
}

/// Prints to the terminal output appending a newline.
// #[macro_export]
// #[macro_pub]
pub macro println {
    () => ($crate::print!("\n")),
    ($($arg:tt)*) => {
        $crate::io::_print(format_args!("{}\n", format_args!($($arg)*)))
    }
}

// #[doc(inline)]
// pub use __print__ as print;

// #[doc(inline)]
// pub use __println__ as println;
