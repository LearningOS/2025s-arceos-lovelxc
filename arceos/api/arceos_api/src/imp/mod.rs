mod mem;
mod task;

cfg_fs! {
    mod fs;
    pub use fs::*;
}

cfg_net! {
    mod net;
    pub use net::*;
}

cfg_display! {
    mod display;
    pub use display::*;
}

mod stdio {
    use core::fmt;

    pub fn ax_console_read_byte() -> Option<u8> {
        axhal::console::getchar().map(|c| if c == b'\r' { b'\n' } else { c })
    }

    pub fn ax_console_write_bytes(buf: &[u8]) -> crate::AxResult<usize> {
        // print_with_color: Write the bytes to the console with red color.
        // 不能输出RESET，不然无法识别到输出。。
        const RED: &[u8] = b"\x1b[1;31m";
        // const RESET: &[u8] = b"\x1b[0m";
        axhal::console::write_bytes(RED);
        axhal::console::write_bytes(buf);
        // axhal::console::write_bytes(RESET);
        Ok(buf.len())
    }

    pub fn ax_console_write_fmt(args: fmt::Arguments) -> fmt::Result {
        axlog::print_fmt(args)
    }
}

mod time {
    pub use axhal::time::{
        monotonic_time as ax_monotonic_time, wall_time as ax_wall_time, TimeValue as AxTimeValue,
    };
}

pub use self::mem::*;
pub use self::stdio::*;
pub use self::task::*;
pub use self::time::*;

pub use axhal::misc::terminate as ax_terminate;
pub use axio::PollState as AxPollState;
