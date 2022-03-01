//! An extremely simple libtock-rs example. Register button events.

#![no_main]
#![no_std]

use core::cell::Cell;
use libtock2::runtime::{set_main, stack_size};
use libtock_platform::{share, subscribe::OneId, DefaultConfig, ErrorCode, Syscalls, Upcall};
use libtock_runtime::TockSyscalls as syscalls;
use ufmt::{uWrite, uwriteln};

set_main! {main}
stack_size! {0x100}

static mut RESULT: u64 = 0x42;
static mut RESULT_REF: &u64 = unsafe { &RESULT };

static HELLO_WORLD: &str = "Hello World!";
static HELLO_WORLD_REF: &&str = &HELLO_WORLD;

fn main() {
    let mut stdout = Console();

    uwriteln!(stdout, "").unwrap();

    uwriteln!(stdout, "Content in Flash").unwrap();
    uwriteln!(stdout, "  HELLO_WORLD      = {}", HELLO_WORLD).unwrap();
    uwriteln!(
        stdout,
        "  &HELLO_WORLD     = {}",
        (&HELLO_WORLD as *const &str) as u64
    )
    .unwrap();
    uwriteln!(stdout, "  *HELLO_WORLD_REF = {}", *HELLO_WORLD_REF).unwrap();
    uwriteln!(
        stdout,
        "  HELLO_WORLD_REF  = {}",
        (HELLO_WORLD_REF as *const &str) as u64
    )
    .unwrap();
    uwriteln!(
        stdout,
        "  &HELLO_WORLD_REF = {}",
        (&HELLO_WORLD_REF as *const &&str) as u64
    )
    .unwrap();
    uwriteln!(stdout, "").unwrap();

    unsafe {
        uwriteln!(stdout, "Content in RAM").unwrap();
        uwriteln!(stdout, "  RESULT      = {}", RESULT).unwrap();
        uwriteln!(stdout, "  &RESULT     = {}", (&RESULT as *const u64) as u64).unwrap();
        uwriteln!(stdout, "  *RESULT_REF = {}", *RESULT_REF).unwrap();
        uwriteln!(
            stdout,
            "  RESULT_REF  = {}",
            (RESULT_REF as *const u64) as u64
        )
        .unwrap();
        uwriteln!(
            stdout,
            "  &RESULT_REF = {}",
            (&RESULT_REF as *const &u64) as u64
        )
        .unwrap();
        uwriteln!(stdout, "").unwrap();
    }
}

struct Console();

impl uWrite for Console {
    type Error = ErrorCode;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let mut buff = s.as_bytes();
        while buff.len() > 0 {
            let written = print(buff)? as usize;
            buff = &buff[written..];
        }
        Ok(())
    }
}

fn print(text: &[u8]) -> Result<u32, ErrorCode> {
    let result: Cell<u32> = Cell::new(0);
    let listener = ConsoleWriteListener(|bytes| result.set(bytes));
    share::scope(|allow_ro| {
        share::scope(|subscribe| {
            syscalls::command(DRIVER_NUM, DRIVER_CHECK, 0, 0).to_result()?;
            syscalls::allow_ro::<DefaultConfig, DRIVER_NUM, WRITE_BUFFER>(allow_ro, text)?;
            syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, WRITE_CALLBACK>(
                subscribe, &listener,
            )?;
            syscalls::command(DRIVER_NUM, CONSOLE_WRITE, text.len() as u32, 0).to_result()?;
            syscalls::yield_wait();
            Ok(0)
        })
    })?;
    Ok(result.get())
}

struct ConsoleWriteListener<F: Fn(u32)>(pub F);

impl<F: Fn(u32)> Upcall<OneId<DRIVER_NUM, WRITE_CALLBACK>> for ConsoleWriteListener<F> {
    fn upcall(&self, bytes: u32, _arg1: u32, _arg2: u32) {
        self.0(bytes)
    }
}

const DRIVER_NUM: u32 = 1;

const DRIVER_CHECK: u32 = 0;
const CONSOLE_WRITE: u32 = 1;

const WRITE_CALLBACK: u32 = 1;

const WRITE_BUFFER: u32 = 1;
