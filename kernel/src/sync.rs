//! Spinlocks for multi-hart critical sections (1.7).

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// Simple test-and-set spinlock. IRQs stay off in the kernel trap path,
/// so we do not nest interrupt disable here.
pub struct SpinLock {
    locked: AtomicBool,
}

pub struct SpinGuard<'a> {
    lock: &'a SpinLock,
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> SpinGuard<'_> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        SpinGuard { lock: self }
    }
}

impl Drop for SpinGuard<'_> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

/// Mutex that parks a value behind a [`SpinLock`].
pub struct Mutex<T> {
    lock: SpinLock,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: SpinLock::new(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        let guard = self.lock.lock();
        MutexGuard {
            _guard: guard,
            data: unsafe { &mut *self.data.get() },
        }
    }
}

pub struct MutexGuard<'a, T> {
    _guard: SpinGuard<'a>,
    data: &'a mut T,
}

impl<T> core::ops::Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> core::ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}
