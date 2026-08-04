//! DeepRoot userspace runtime — syscall wrappers.

#![no_std]

use deeproot_abi::syscall::*;

pub mod sys {
    use super::*;

    #[inline(always)]
    unsafe fn ecall(nr: usize, a0: usize, a1: usize, a2: usize, a3: usize) -> isize {
        let mut ret: isize;
        core::arch::asm!(
            "ecall",
            in("a7") nr,
            inout("a0") a0 => ret,
            in("a1") a1,
            in("a2") a2,
            in("a3") a3,
            options(nostack),
        );
        ret
    }

    pub fn debug_write(s: &str) -> isize {
        unsafe { ecall(SYS_DEBUG_WRITE, s.as_ptr() as usize, s.len(), 0, 0) }
    }

    pub fn yield_now() -> isize {
        unsafe { ecall(SYS_YIELD, 0, 0, 0, 0) }
    }

    pub fn ipc_call(ep_slot: usize, label: u64, word0: u64) -> isize {
        unsafe { ecall(SYS_IPC_CALL, ep_slot, label as usize, word0 as usize, 0) }
    }

    pub fn ipc_recv(badge: u64) -> isize {
        unsafe { ecall(SYS_IPC_RECV, badge as usize, 0, 0, 0) }
    }

    pub fn ipc_reply(badge: u64, label: u64, word0: u64) -> isize {
        unsafe { ecall(SYS_IPC_REPLY, badge as usize, label as usize, word0 as usize, 0) }
    }

    pub fn exit(code: usize) -> ! {
        unsafe {
            let _ = ecall(SYS_EXIT, code, 0, 0, 0);
        }
        loop {
            unsafe {
                core::arch::asm!("wfi");
            }
        }
    }
}
