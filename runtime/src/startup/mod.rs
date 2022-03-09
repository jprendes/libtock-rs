//! Runtime components related to process startup.

// Include the correct `start` symbol (the program entry point) for the
// architecture.
#[cfg(target_arch = "arm")]
core::arch::global_asm!(include_str!("asm_arm.s"));
#[cfg(target_arch = "riscv32")]
core::arch::global_asm!(include_str!("asm_riscv32.s"));

/// `set_main!` is used to tell `libtock_runtime` where the process binary's
/// `main` function is. The process binary's `main` function must have the
/// signature `FnOnce() -> T`, where T is some concrete type that implements
/// `libtock_platform::Termination`.
///
/// # Example
/// ```
/// libtock_runtime::set_main!{main};
///
/// fn main() -> () { /* Omitted */ }
/// ```
// set_main! generates a function called `libtock_unsafe_main`, which is called
// by `rust_start`. The function has `unsafe` in its name because implementing
// it is `unsafe` (it *must* have the signature `libtock_unsafe_main() -> !`),
// but there is no way to enforce the use of `unsafe` through the type system.
// This function calls the client-provided function, which enforces its type
// signature.
#[macro_export]
macro_rules! set_main {
    {$name:ident} => {
        #[no_mangle]
        fn libtock_unsafe_main() -> ! {
            use libtock_runtime::TockSyscalls;
            let res = $name();
            #[allow(unreachable_code)] // so that fn main() -> ! does not produce a warning.
            libtock_platform::Termination::complete::<TockSyscalls>(res)
        }
    }
}

/// Executables must specify their stack size by using the `stack_size!` macro.
/// It takes a single argument, the desired stack size in bytes. Example:
/// ```
/// stack_size!{0x400}
/// ```
// stack_size works by putting a symbol equal to the size of the stack in the
// .stack_buffer section. The linker script uses the .stack_buffer section to
// size the stack. flash.sh looks for the symbol by name (hence #[no_mangle]) to
// determine the size of the stack to pass to elf2tab.
#[macro_export]
macro_rules! stack_size {
    {$size:expr} => {
        #[no_mangle]
        #[link_section = ".stack_buffer"]
        pub static mut STACK_MEMORY: [u8; $size] = [0; $size];
    }
}

mod console;
use crate::{print, println};

#[repr(C)]
#[derive(ufmt::derive::uDebug)]
struct hdr {
    //  0: Offset of GOT symbols in flash from the start of the application
    //     binary.
    got_sym_start: u32,
    //  4: Offset of where the GOT section needs to be placed in memory from the
    //     start of the application's memory region.
    got_start: u32,
    //  8: Size of GOT section.
    got_size: u32,
    // 12: Offset of data symbols in flash from the start of the application
    //     binary.
    data_sym_start: u32,
    // 16: Offset of where the data section needs to be placed in memory from the
    //     start of the application's memory region.
    data_start: u32,
    // 20: Size of data section.
    data_size: u32,
    // 24: Offset of where the BSS section needs to be placed in memory from the
    //     start of the application's memory region.
    bss_start: u32,
    // 28: Size of BSS section.
    bss_size: u32,
    // 32: First address offset after program flash, where elf2tab places
    //     .rel.data section
    reldata_start: u32,
    // 36: The size of the stack requested by this application.
    stack_size: u32,
}

#[repr(C)]
struct reldata {
    // Number of relative addresses.
    len: u32,
    // Array of offsets of the address to be updated relative to the start of the
    // application's memory region. Each address at these offsets needs to be
    // adjusted to be a fixed address relative to the start of the app's actual
    // flash or RAM start address.
    data: rela,
}

#[repr(C)]
#[derive(ufmt::derive::uDebug)]
struct rela {
    offset: u32,
    info: u32,
    addend: u32,
}

unsafe fn make_slice<'a, T>(base: u32, offset: u32, num_bytes: u32) -> &'a [T] {
    unsafe {
        core::slice::from_raw_parts((base + offset) as *const T, num_bytes as usize / core::mem::size_of::<T>())
    }
}

unsafe fn make_slice_mut<'a, T>(base: u32, offset: u32, num_bytes: u32) -> &'a mut [T] {
    unsafe {
        core::slice::from_raw_parts_mut((base + offset) as *mut T, num_bytes as usize / core::mem::size_of::<T>())
    }
}

// rust_start is the first Rust code to execute in the process. It is called
// from start, which is written directly in assembly.
#[no_mangle]
extern "C" fn rust_start(app_start: u32, mem_start: u32) -> ! {
    unsafe {
        let myhdr : &hdr = &*(app_start as *const hdr);

        let got_sram = make_slice_mut::<u32>(mem_start, myhdr.got_start, myhdr.got_size);
        let got_flash = make_slice::<u32>(app_start, myhdr.got_sym_start, myhdr.got_size);

        for (sram_entry, flash_entry) in got_sram.iter_mut().zip(got_flash.iter()) {
            *sram_entry = if (flash_entry & 0x80000000) == 0 {
                flash_entry + mem_start
            } else {
                (flash_entry ^ 0x80000000) + app_start
            };
        }

        let data_sram = make_slice_mut::<u8>(mem_start, myhdr.data_start, myhdr.data_size);
        let data_flash = make_slice::<u8>(app_start, myhdr.data_sym_start, myhdr.data_size);

        data_sram.clone_from_slice(data_flash);

        let bss_sram = make_slice_mut::<u8>(mem_start, myhdr.bss_start, myhdr.bss_size);

        bss_sram.fill(0);

        let rd : &reldata = &*((myhdr.reldata_start + app_start) as *const reldata);
        let rd_data = make_slice::<rela>(app_start, myhdr.reldata_start + 4, rd.len);

        println!();
        println!();
        
        println!("mem_start = {:#x}", mem_start);
        println!("app_start = {:#x}", app_start);
        println!();

        println!("Copy data: {:#x} bytes from {:#?} to {:#?}", data_sram.len(), data_flash.as_ptr(), data_sram.as_ptr());
        println!("Zero init data: {:#x} bytes from {:#?}", bss_sram.len(), bss_sram.as_ptr());
        println!();

        println!("Relocating things!");
        println!();

        println!("myhdr = {:#?}", myhdr);
        println!();

        for rela{offset,..} in rd_data {
            let target = (offset + mem_start) as *mut u32;
            print!("Relocating {:#?} ({:#x}, target={:#x}) : {:#x} -> ", offset, offset, offset + mem_start, *target);
            *target = if (*target & 0x80000000) == 0 {
                *target + mem_start
            } else {
                //core::ptr::write_volatile(target, (core::ptr::read_volatile(target) ^ 0x80000000) + app_start);
                (*target ^ 0x80000000) + app_start
            };
            println!("{:#x}", *target);
        }

        println!("Done relocating things!");
    }

    extern "Rust" {
        fn libtock_unsafe_main() -> !;
    }
    unsafe {
        libtock_unsafe_main();
    }
}
