//! An extremely simple libtock-rs example. Register button events.

#![no_main]
#![no_std]

use libtock2::println;
use libtock2::runtime::{set_main, stack_size};
use ufmt::derive::uDebug;

set_main! {main}
stack_size! {0x300}

static mut RESULT: u64 = 0x42;
static mut RESULT_REF: *const u64 = unsafe { core::ptr::addr_of!(RESULT) };

static HELLO_WORLD: &str = "Hello World!";
static mut HELLO_WORLD_REF: *const &str = core::ptr::addr_of!(HELLO_WORLD);

#[derive(uDebug)]
struct Test<'a> {
    a: u32,
    b: u64,
    c: &'a str,
}

fn main() {
    unsafe {
        println!();

        let test = Test {
            a: 123,
            b: 456,
            c: "Hello World!",
        };

        println!("test.c.as_ptr() = {:#x}", test.c.as_ptr() as u32);
        println!("HELLO_WORLD.as_ptr() = {:#x}", HELLO_WORLD.as_ptr() as u32);
        println!("test = {:#?}", test);
        println!();

        println!("Content in Flash");
        println!(
            "  &HELLO_WORLD     = {:#x}",
            core::ptr::addr_of!(HELLO_WORLD) as u32
        );
        println!(
            "  &HELLO_WORLD_REF = {:#x}",
            core::ptr::addr_of!(HELLO_WORLD_REF) as u32
        );
        println!();
        println!(
            "  HELLO_WORLD_REF  = {:#x}",
            HELLO_WORLD_REF as *const &str as u32
        );
        println!();
        println!("  HELLO_WORLD      = {}", HELLO_WORLD);
        println!("  *HELLO_WORLD_REF = {}", *(HELLO_WORLD_REF as *const &str));
        println!();

        println!("Content in RAM");
        println!("  &RESULT     = {:#x}", core::ptr::addr_of!(RESULT) as u32);
        println!(
            "  &RESULT_REF = {:#x}",
            core::ptr::addr_of!(RESULT_REF) as u32
        );
        println!();
        println!("  RESULT_REF  = {:#x}", RESULT_REF as u32);
        println!();
        println!("  RESULT      = {:#x}", RESULT);
        println!("  *RESULT_REF = {:#x}", *RESULT_REF);
        println!();
    }
}
