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

    pub fn debug_write_bytes(buf: &[u8]) -> isize {
        unsafe { ecall(SYS_DEBUG_WRITE, buf.as_ptr() as usize, buf.len(), 0, 0) }
    }

    pub fn debug_read_byte() -> isize {
        unsafe { ecall(SYS_DEBUG_READ, 0, 0, 0, 0) }
    }

    pub fn yield_now() -> isize {
        unsafe { ecall(SYS_YIELD, 0, 0, 0, 0) }
    }

    pub fn spawn(blob_id: usize) -> isize {
        unsafe { ecall(SYS_SPAWN, blob_id, 0, 0, 0) }
    }

    pub fn fs_list() -> isize {
        unsafe { ecall(SYS_FS_LIST, 0, 0, 0, 0) }
    }

    pub fn fs_list_path(path: &[u8]) -> isize {
        unsafe { ecall(SYS_FS_LIST, path.as_ptr() as usize, path.len(), 0, 0) }
    }

    pub fn fs_cat(path: &[u8]) -> isize {
        unsafe { ecall(SYS_FS_CAT, path.as_ptr() as usize, path.len(), 0, 0) }
    }

    pub fn fs_write(path: &[u8], data: &[u8]) -> isize {
        unsafe {
            ecall(
                SYS_FS_WRITE,
                path.as_ptr() as usize,
                path.len(),
                data.as_ptr() as usize,
                data.len(),
            )
        }
    }

    pub fn fs_append(path: &[u8], data: &[u8]) -> isize {
        unsafe {
            ecall(
                SYS_FS_APPEND,
                path.as_ptr() as usize,
                path.len(),
                data.as_ptr() as usize,
                data.len(),
            )
        }
    }

    pub fn fs_mkdir(path: &[u8]) -> isize {
        unsafe { ecall(SYS_FS_MKDIR, path.as_ptr() as usize, path.len(), 0, 0) }
    }

    pub fn fs_rmdir(path: &[u8]) -> isize {
        unsafe { ecall(SYS_FS_RMDIR, path.as_ptr() as usize, path.len(), 0, 0) }
    }

    pub fn fs_unlink(path: &[u8]) -> isize {
        unsafe { ecall(SYS_FS_UNLINK, path.as_ptr() as usize, path.len(), 0, 0) }
    }

    pub fn chdir(path: &[u8]) -> isize {
        unsafe { ecall(SYS_CHDIR, path.as_ptr() as usize, path.len(), 0, 0) }
    }

    pub fn getcwd(buf: &mut [u8]) -> isize {
        unsafe { ecall(SYS_GETCWD, buf.as_mut_ptr() as usize, buf.len(), 0, 0) }
    }

    /// Spawn path ELF as IPC server; mint endpoint into caller. → cap slot
    pub fn spawn_server(path: &[u8], badge: u64) -> isize {
        unsafe {
            ecall(
                SYS_SPAWN_SERVER,
                path.as_ptr() as usize,
                path.len(),
                badge as usize,
                0,
            )
        }
    }

    pub fn module_list() -> isize {
        unsafe { ecall(SYS_MODULE_LIST, 0, 0, 0, 0) }
    }

    pub fn fs_cp(src: &[u8], dst: &[u8]) -> isize {
        unsafe {
            ecall(
                SYS_FS_CP,
                src.as_ptr() as usize,
                src.len(),
                dst.as_ptr() as usize,
                dst.len(),
            )
        }
    }

    pub fn open(path: &[u8], flags: u32) -> isize {
        unsafe { ecall(SYS_OPEN, path.as_ptr() as usize, path.len(), flags as usize, 0) }
    }

    pub fn close(fd: usize) -> isize {
        unsafe { ecall(SYS_CLOSE, fd, 0, 0, 0) }
    }

    pub fn fd_read(fd: usize, buf: &mut [u8]) -> isize {
        unsafe { ecall(SYS_FD_READ, fd, buf.as_mut_ptr() as usize, buf.len(), 0) }
    }

    pub fn fd_write(fd: usize, buf: &[u8]) -> isize {
        unsafe { ecall(SYS_FD_WRITE, fd, buf.as_ptr() as usize, buf.len(), 0) }
    }

    pub fn lseek(fd: usize, offset: isize, whence: usize) -> isize {
        unsafe { ecall(SYS_LSEEK, fd, offset as usize, whence, 0) }
    }

    pub fn sleep_ms(ms: u64) -> isize {
        unsafe { ecall(SYS_SLEEP_MS, ms as usize, 0, 0, 0) }
    }

    pub fn ledger_dump() -> isize {
        unsafe { ecall(SYS_LEDGER_DUMP, 0, 0, 0, 0) }
    }

    pub fn cap_dump() -> isize {
        unsafe { ecall(SYS_CAP_DUMP, 0, 0, 0, 0) }
    }

    /// Resolve registered service name → Endpoint cap slot in this task.
    pub fn service_lookup(name: &[u8]) -> isize {
        unsafe { ecall(SYS_SERVICE_LOOKUP, name.as_ptr() as usize, name.len(), 0, 0) }
    }

    pub const O_RDONLY: u32 = 0;
    pub const O_WRONLY: u32 = 1;
    pub const O_RDWR: u32 = 2;
    pub const O_CREAT: u32 = 0x40;
    pub const O_TRUNC: u32 = 0x200;
    pub const O_APPEND: u32 = 0x400;

    pub fn exec(path: &[u8]) -> isize {
        unsafe { ecall(SYS_EXEC, path.as_ptr() as usize, path.len(), 0, 0) }
    }

    pub fn time_ms() -> u64 {
        let v = unsafe { ecall(SYS_TIME, 0, 0, 0, 0) };
        if v < 0 {
            0
        } else {
            v as u64
        }
    }

    pub fn wait(id: usize) -> isize {
        unsafe { ecall(SYS_WAIT, id, 0, 0, 0) }
    }

    pub fn pipe() -> isize {
        unsafe { ecall(SYS_PIPE, 0, 0, 0, 0) }
    }

    pub fn pipe_close(id: usize) -> isize {
        unsafe { ecall(SYS_PIPE_CLOSE, id, 0, 0, 0) }
    }

    pub fn pipe_read(id: usize, buf: &mut [u8]) -> isize {
        unsafe { ecall(SYS_PIPE_READ, id, buf.as_mut_ptr() as usize, buf.len(), 0) }
    }

    pub fn pipe_write(id: usize, buf: &[u8]) -> isize {
        unsafe { ecall(SYS_PIPE_WRITE, id, buf.as_ptr() as usize, buf.len(), 0) }
    }

    pub fn task_stdout(task: usize, pipe_or_console: usize) -> isize {
        unsafe { ecall(SYS_TASK_STDOUT, task, pipe_or_console, 0, 0) }
    }

    pub const STDOUT_CONSOLE: usize = deeproot_abi::syscall::STDOUT_CONSOLE;

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
