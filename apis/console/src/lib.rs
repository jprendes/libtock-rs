#![no_std]

use libtock_platform::{share, subscribe::OneId, DefaultConfig, Syscalls, Upcall};
use core::cell::Cell;
pub use libtock_platform::ErrorCode;

pub struct Console<S: Syscalls>(S);

impl<S: Syscalls> Console<S> {

    pub fn print(text: &[u8]) -> Result<u32, ErrorCode> {
        if text.len() == 0 { return Ok(0); }
        let result: Cell<u32> = Cell::new(0);
        let listener = ConsoleWriteListener(|bytes| result.set(bytes));
        share::scope(|allow_ro| {
            share::scope(|subscribe| {
                S::command(DRIVER_NUM, DRIVER_CHECK, 0, 0).to_result()?;
                S::allow_ro::<DefaultConfig, DRIVER_NUM, WRITE_BUFFER>(allow_ro, text)?;
                S::subscribe::<_, _, DefaultConfig, DRIVER_NUM, WRITE_CALLBACK>(
                    subscribe, &listener,
                )?;
                S::command(DRIVER_NUM, CONSOLE_WRITE, text.len() as u32, 0).to_result()?;
                S::yield_wait();
                Ok(0)
            })
        })?;
        Ok(result.get())
    }

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
