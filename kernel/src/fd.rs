//! Per-task file descriptor table (1.11.1).
//!
//! Teaching fds: path-backed open/read/write/close (+ lseek). Not POSIX;
//! max 8 fds per task; paths snapshotted at open.

use crate::fs;
use crate::sched::MAX_UTASKS;
use crate::sync::SpinLock;
use crate::vfs::FILE_MAX;

pub const MAX_FDS: usize = 8;

pub const O_RDONLY: u32 = 0;
pub const O_WRONLY: u32 = 1;
pub const O_RDWR: u32 = 2;
pub const O_CREAT: u32 = 0x40;
pub const O_TRUNC: u32 = 0x200;
pub const O_APPEND: u32 = 0x400;

const ACCESS_MASK: u32 = 0x3;

#[derive(Clone, Copy)]
struct Fd {
    used: bool,
    flags: u32,
    offset: usize,
    cwd: usize,
    path: [u8; 96],
    path_len: usize,
    len: usize,
}

impl Fd {
    const fn empty() -> Self {
        Self {
            used: false,
            flags: 0,
            offset: 0,
            cwd: 0,
            path: [0; 96],
            path_len: 0,
            len: 0,
        }
    }
}

struct Tables {
    fds: [[Fd; MAX_FDS]; MAX_UTASKS],
}

static LOCK: SpinLock = SpinLock::new();
static mut TABLES: Tables = Tables {
    fds: [[Fd::empty(); MAX_FDS]; MAX_UTASKS],
};

fn tables() -> &'static mut Tables {
    unsafe { &mut *core::ptr::addr_of_mut!(TABLES) }
}

pub fn clear_task(sched_id: usize) {
    if sched_id >= MAX_UTASKS {
        return;
    }
    let _g = LOCK.lock();
    let t = tables();
    for fd in t.fds[sched_id].iter_mut() {
        *fd = Fd::empty();
    }
}

fn access_ok_read(flags: u32) -> bool {
    let a = flags & ACCESS_MASK;
    a == O_RDONLY || a == O_RDWR
}

fn access_ok_write(flags: u32) -> bool {
    let a = flags & ACCESS_MASK;
    a == O_WRONLY || a == O_RDWR
}

/*
 * open - allocate an fd for @path relative to @cwd
 *
 * Returns fd index, or None on failure.
 */
pub fn open(sched_id: usize, cwd: usize, path: &str, flags: u32) -> Option<usize> {
    if sched_id >= MAX_UTASKS || path.is_empty() || path.len() >= 96 {
        return None;
    }
    let want_write = access_ok_write(flags);
    let want_read = access_ok_read(flags);
    if !want_read && !want_write {
        return None;
    }

    let exists = fs::file_len(cwd, path).is_some();
    if !exists {
        if flags & O_CREAT == 0 {
            return None;
        }
        if !want_write {
            return None;
        }
        if !fs::write_path(cwd, path, b"") {
            return None;
        }
    } else if flags & O_TRUNC != 0 && want_write {
        if !fs::write_path(cwd, path, b"") {
            return None;
        }
    }

    let len = fs::file_len(cwd, path).unwrap_or(0);
    let _g = LOCK.lock();
    let t = tables();
    let slot = t.fds[sched_id].iter().position(|f| !f.used)?;
    let f = &mut t.fds[sched_id][slot];
    *f = Fd::empty();
    f.used = true;
    f.flags = flags;
    f.cwd = cwd;
    f.path_len = path.len();
    f.path[..path.len()].copy_from_slice(path.as_bytes());
    f.len = len;
    f.offset = if flags & O_APPEND != 0 { len } else { 0 };
    Some(slot)
}

pub fn close(sched_id: usize, fd: usize) -> bool {
    if sched_id >= MAX_UTASKS || fd >= MAX_FDS {
        return false;
    }
    let _g = LOCK.lock();
    let f = &mut tables().fds[sched_id][fd];
    if !f.used {
        return false;
    }
    *f = Fd::empty();
    true
}

pub fn read(sched_id: usize, fd: usize, out: &mut [u8]) -> Option<usize> {
    if sched_id >= MAX_UTASKS || fd >= MAX_FDS || out.is_empty() {
        return None;
    }
    let (cwd, path_buf, path_len, offset, flags, file_len) = {
        let _g = LOCK.lock();
        let f = &tables().fds[sched_id][fd];
        if !f.used || !access_ok_read(f.flags) {
            return None;
        }
        (
            f.cwd,
            f.path,
            f.path_len,
            f.offset,
            f.flags,
            f.len,
        )
    };
    let _ = flags;
    if offset >= file_len {
        return Some(0);
    }
    let path = core::str::from_utf8(&path_buf[..path_len]).ok()?;
    let mut scratch = [0u8; FILE_MAX];
    let n = fs::read_bytes(cwd, path, &mut scratch)?;
    if offset >= n {
        return Some(0);
    }
    let take = out.len().min(n - offset);
    out[..take].copy_from_slice(&scratch[offset..offset + take]);
    let _g = LOCK.lock();
    let f = &mut tables().fds[sched_id][fd];
    if f.used {
        f.offset = offset + take;
        f.len = n;
    }
    Some(take)
}

pub fn write(sched_id: usize, fd: usize, data: &[u8]) -> Option<usize> {
    if sched_id >= MAX_UTASKS || fd >= MAX_FDS {
        return None;
    }
    let (cwd, path_buf, path_len, offset, flags) = {
        let _g = LOCK.lock();
        let f = &tables().fds[sched_id][fd];
        if !f.used || !access_ok_write(f.flags) {
            return None;
        }
        (f.cwd, f.path, f.path_len, f.offset, f.flags)
    };
    let path = core::str::from_utf8(&path_buf[..path_len]).ok()?;

    let ok = if flags & O_APPEND != 0 {
        fs::append_path(cwd, path, data)
    } else if offset == 0 {
        fs::write_path(cwd, path, data)
    } else {
        /* RMW for mid-file write (teaching; FILE_MAX capped). */
        let mut buf = [0u8; FILE_MAX];
        let old = fs::read_bytes(cwd, path, &mut buf).unwrap_or(0);
        if offset > old || offset + data.len() > FILE_MAX {
            return None;
        }
        buf[offset..offset + data.len()].copy_from_slice(data);
        let new_len = old.max(offset + data.len());
        fs::write_path(cwd, path, &buf[..new_len])
    };
    if !ok {
        return None;
    }
    let new_len = fs::file_len(cwd, path).unwrap_or(0);
    let _g = LOCK.lock();
    let f = &mut tables().fds[sched_id][fd];
    if f.used {
        f.len = new_len;
        f.offset = if flags & O_APPEND != 0 {
            new_len
        } else {
            offset + data.len()
        };
    }
    Some(data.len())
}

pub fn lseek(sched_id: usize, fd: usize, offset: isize, whence: usize) -> Option<usize> {
    if sched_id >= MAX_UTASKS || fd >= MAX_FDS {
        return None;
    }
    let _g = LOCK.lock();
    let f = &mut tables().fds[sched_id][fd];
    if !f.used {
        return None;
    }
    let base = match whence {
        0 => 0usize, /* SEEK_SET */
        1 => f.offset,
        2 => f.len, /* SEEK_END */
        _ => return None,
    };
    let next = if offset < 0 {
        base.checked_sub((-offset) as usize)?
    } else {
        base.checked_add(offset as usize)?
    };
    f.offset = next;
    Some(next)
}

    #[allow(dead_code)]
    pub fn path_of(sched_id: usize, fd: usize, out: &mut [u8]) -> Option<usize> {
    if sched_id >= MAX_UTASKS || fd >= MAX_FDS {
        return None;
    }
    let _g = LOCK.lock();
    let f = &tables().fds[sched_id][fd];
    if !f.used {
        return None;
    }
    let n = f.path_len.min(out.len());
    out[..n].copy_from_slice(&f.path[..n]);
    Some(n)
}
