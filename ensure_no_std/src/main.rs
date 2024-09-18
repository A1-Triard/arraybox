#![feature(start)]

#![deny(warnings)]

#![no_std]

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

mod no_std {
    use core::panic::PanicInfo;
    use exit_no_std::exit;

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        exit(99)
    }

    #[no_mangle]
    extern "C" fn rust_eh_personality() { }
}

use arraybox::ArrayBox;

trait Int {
    fn value(&self) -> i32;
}

struct IntValue(i32);

impl Int for IntValue {
    fn value(&self) -> i32 { self.0 }
}

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let a: ArrayBox<'static, dyn Int, IntValue> = ArrayBox::new(IntValue(7i32));
    assert_eq!(a.value(), 7i32);
    0
}
