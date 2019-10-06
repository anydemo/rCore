#![no_std]
#![feature(lang_items)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::alloc::GlobalAlloc;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::panic::PanicInfo;
use rcore_api::RcoreAPI;

#[no_mangle]
#[cfg_attr(target_arch = "mips", export_name = "__start")]
pub extern "C" fn _start(api: &'static RcoreAPI) {
    unsafe {
        RCORE_API = Some(api);
    }
    log::set_logger(api.logger).unwrap();
    info!("Hello from kernel module!");
    let a = vec![1, 2, 3];
    for i in a {
        println!("{}", i);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);
    loop {}
}

#[lang = "oom"]
fn oom(_: Layout) -> ! {
    panic!("out of memory");
}

#[no_mangle]
fn abort() {
    panic!("abort");
}

static mut RCORE_API: Option<&'static RcoreAPI> = None;

fn rcore_api() -> &'static RcoreAPI {
    unsafe { RCORE_API.expect("rcore api is unavailable") }
}

/// Allocator which uses the RcoreAPI allocator
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        rcore_api().allocator.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        rcore_api().allocator.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn print(args: core::fmt::Arguments) {
    (rcore_api().print)(args);
}
