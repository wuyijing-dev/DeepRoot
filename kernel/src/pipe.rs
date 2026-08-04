//! In-kernel byte pipes for shell `|` / stdout capture (1.8).

use core::cell::UnsafeCell;

use crate::sync::SpinLock;

pub const MAX_PIPES: usize = 4;
pub const PIPE_CAP: usize = 512;

struct Pipe {
    live: bool,
    buf: [u8; PIPE_CAP],
    head: usize,
    tail: usize,
    len: usize,
}

impl Pipe {
    const fn empty() -> Self {
        Self {
            live: false,
            buf: [0; PIPE_CAP],
            head: 0,
            tail: 0,
            len: 0,
        }
    }
}

struct Table {
    pipes: [Pipe; MAX_PIPES],
}

struct Cell(UnsafeCell<Table>);
unsafe impl Sync for Cell {}

static LOCK: SpinLock = SpinLock::new();
static TABLE: Cell = Cell(UnsafeCell::new(Table {
    pipes: [const { Pipe::empty() }; MAX_PIPES],
}));

fn table() -> &'static mut Table {
    unsafe { &mut *TABLE.0.get() }
}

pub fn create() -> Option<usize> {
    let _g = LOCK.lock();
    let t = table();
    let id = t.pipes.iter().position(|p| !p.live)?;
    t.pipes[id] = Pipe::empty();
    t.pipes[id].live = true;
    Some(id)
}

pub fn close(id: usize) {
    let _g = LOCK.lock();
    if id < MAX_PIPES {
        table().pipes[id] = Pipe::empty();
    }
}

pub fn write(id: usize, data: &[u8]) -> usize {
    let _g = LOCK.lock();
    if id >= MAX_PIPES || !table().pipes[id].live {
        return 0;
    }
    let p = &mut table().pipes[id];
    let mut n = 0usize;
    for &b in data {
        if p.len >= PIPE_CAP {
            break;
        }
        p.buf[p.tail] = b;
        p.tail = (p.tail + 1) % PIPE_CAP;
        p.len += 1;
        n += 1;
    }
    n
}

pub fn read(id: usize, out: &mut [u8]) -> usize {
    let _g = LOCK.lock();
    if id >= MAX_PIPES || !table().pipes[id].live {
        return 0;
    }
    let p = &mut table().pipes[id];
    let mut n = 0usize;
    while n < out.len() && p.len > 0 {
        out[n] = p.buf[p.head];
        p.head = (p.head + 1) % PIPE_CAP;
        p.len -= 1;
        n += 1;
    }
    n
}
