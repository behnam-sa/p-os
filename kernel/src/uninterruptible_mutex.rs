use core::{
    fmt,
    ops::{Deref, DerefMut},
};
use spin::{Mutex, MutexGuard};
use x86_64::instructions::interrupts;

pub struct UninterruptibleMutex<T: ?Sized> {
    mutex: Mutex<T>,
}

pub struct UninterruptibleMutexGuard<'a, T: 'a + ?Sized> {
    lock: MutexGuard<'a, T>,
    interrupt_guard: InterruptGuard,
}

pub struct InterruptGuard {
    were_interrupts_enabled: bool,
}

impl<T> UninterruptibleMutex<T> {
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        Self {
            mutex: Mutex::new(value),
        }
    }
}

impl<T: ?Sized> UninterruptibleMutex<T> {
    #[inline(always)]
    pub fn lock(&self) -> UninterruptibleMutexGuard<T> {
        let interrupt_guard = InterruptGuard::new();

        UninterruptibleMutexGuard {
            lock: self.mutex.lock(),
            interrupt_guard,
        }
    }

    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.mutex.is_locked()
    }

    #[inline(always)]
    pub unsafe fn force_unlock(&self) {
        self.mutex.force_unlock()
    }

    #[inline(always)]
    pub fn try_lock(&self) -> Option<UninterruptibleMutexGuard<T>> {
        let interrupt_guard = InterruptGuard::new();

        match self.mutex.try_lock() {
            Some(lock) => Some(UninterruptibleMutexGuard {
                lock,
                interrupt_guard,
            }),
            None => None,
        }
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        self.mutex.get_mut()
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for UninterruptibleMutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.mutex, f)
    }
}

impl<T: ?Sized + Default> Default for UninterruptibleMutex<T> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T> From<T> for UninterruptibleMutex<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for UninterruptibleMutexGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for UninterruptibleMutexGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized> Deref for UninterruptibleMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &*self.lock
    }
}

impl<'a, T: ?Sized> DerefMut for UninterruptibleMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.lock
    }
}

impl InterruptGuard {
    pub fn new() -> InterruptGuard {
        let were_interrupts_enabled = interrupts::are_enabled();

        if were_interrupts_enabled {
            interrupts::disable();
        }

        Self {
            were_interrupts_enabled,
        }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if self.were_interrupts_enabled {
            interrupts::enable();
        }
    }
}
