// SPDX-License-Identifier: GPL-3.0-only
//! Exit Syscall Handler
//!
//! Authors: MarioS271

use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::state::kstate::KSTATE;

pub fn handler(exit_code: u32) -> ! {
    let pid = KSTATE.sched.active_pid();
    // Safety: syscalls get inited in stage 2, this only needs stage 0
    unsafe { KSTATE.sched.remove_process(pid) };

    crate::kdebug!("Process {} exited with code {}", pid, exit_code);

    // TODO: properly exit
    kernel_panic(
        PanicCode::InitProcessDied,
        "Process exited"
    );
}
