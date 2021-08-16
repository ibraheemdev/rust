use crate::cell::UnsafeCell;
use crate::sys::c;

pub struct RwLock {
    inner: UnsafeCell<c::SRWLOCK>,
}

pub type MovableRwLock = RwLock;

unsafe impl Send for RwLock {}
unsafe impl Sync for RwLock {}

impl RwLock {
    pub const fn new() -> RwLock {
        RwLock { inner: UnsafeCell::new(c::SRWLOCK_INIT) }
    }
    #[inline]
    pub unsafe fn read(&self) {
        c::AcquireSRwLockShared(self.inner.get())
    }
    #[inline]
    pub unsafe fn try_read(&self) -> bool {
        c::TryAcquireSRwLockShared(self.inner.get()) != 0
    }
    #[inline]
    pub unsafe fn write(&self) {
        c::AcquireSRwLockExclusive(self.inner.get())
    }
    #[inline]
    pub unsafe fn try_write(&self) -> bool {
        c::TryAcquireSRwLockExclusive(self.inner.get()) != 0
    }
    #[inline]
    pub unsafe fn read_unlock(&self) {
        c::ReleaseSRwLockShared(self.inner.get())
    }
    #[inline]
    pub unsafe fn write_unlock(&self) {
        c::ReleaseSRwLockExclusive(self.inner.get())
    }

    #[inline]
    pub unsafe fn destroy(&self) {
        // ...
    }
}
