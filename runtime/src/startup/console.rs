use libtock_console as console;
use ufmt::uWrite;
pub use console::ErrorCode;
pub use ufmt;

pub struct Console;

impl uWrite for Console {
    type Error = ErrorCode;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        let mut buff = s.as_bytes();
        while buff.len() > 0 {
            let written = console::Console::<crate::TockSyscalls>::print(buff)? as usize;
            buff = &buff[written..];
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! write {
    ($($arg:tt)*) => ({ $crate::startup::console::ufmt::uwrite!($crate::startup::console::Console, $($arg)*) });
}

#[macro_export]
macro_rules! writeln {
    () => ({ $crate::writeln!("") });
    ($($arg:tt)*) => ({ $crate::startup::console::ufmt::uwriteln!($crate::startup::console::Console, $($arg)*) });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({ $crate::write!($($arg)*).unwrap() });
}

#[macro_export]
macro_rules! println {
    () => ({ $crate::writeln!("").unwrap() });
    ($($arg:tt)*) => ({ $crate::writeln!($($arg)*).unwrap() });
}