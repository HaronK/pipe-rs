use core::cell::UnsafeCell;
use core::ptr;

use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    next: UnsafeCell<*mut Node<T>>,
    value: Option<T>,
}

/// Pipe writer trait
pub trait PipeWriter<T> {
    /// Write method
    fn write(&self, elt: T);
}

/// Pipe reader trait
pub trait PipeReader<T> {
    /// Read method
    fn read(&self) -> Option<T>;
}

/// Pipe structure
#[derive(Debug)]
pub struct Pipe<T> {
    writer_head: AtomicPtr<Node<T>>,
    writer_tail: UnsafeCell<*mut Node<T>>,
    writer_finished: bool,

    reader_head: AtomicPtr<Node<T>>,
}

impl<T> Default for Pipe<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Send> Send for Pipe<T> {}
unsafe impl<T: Send> Sync for Pipe<T> {}

impl<T> Node<T> {
    #[inline]
    unsafe fn new(v: T) -> *mut Node<T> {
        Box::into_raw(Box::new(Node {
            next: UnsafeCell::new(ptr::null_mut()),
            value: Some(v),
        }))
    }
}

impl<T> Pipe<T> {
    /// Constructor
    #[inline]
    pub fn new() -> Pipe<T> {
        Pipe {
            writer_head: AtomicPtr::new(ptr::null_mut()),
            writer_tail: UnsafeCell::new(ptr::null_mut()),
            writer_finished: false,
            reader_head: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl<T> Drop for Pipe<T> {
    fn drop(&mut self) {
        unsafe {
            // clean writer's subqueue
            let mut cur = self.writer_head.load(Ordering::Relaxed);
            while !cur.is_null() {
                let next = *(*cur).next.get();
                let _: Box<Node<T>> = Box::from_raw(cur);
                cur = next;
            }
            // clean reader's subqueue
            cur = self.reader_head.load(Ordering::Relaxed);
            while !cur.is_null() {
                let next = *(*cur).next.get();
                let _: Box<Node<T>> = Box::from_raw(cur);
                cur = next;
            }
        }
    }
}

impl<T> PipeWriter<T> for Pipe<T> {
    fn write(&self, elt: T) {
        assert!(!self.writer_finished);

        unsafe {
            let node = Node::new(elt);

            let mut head = self.writer_head.swap(ptr::null_mut(), Ordering::AcqRel);
            if head.is_null() {
                head = node;
            } else {
                let tail = *self.writer_tail.get();
                *(*tail).next.get() = node;
            }
            *self.writer_tail.get() = node;

            let reader_head = self.reader_head.load(Ordering::Acquire);
            if reader_head.is_null() {
                self.reader_head.store(head, Ordering::Release);
                return;
            }

            self.writer_head.store(head, Ordering::Release);
        }
    }
}

impl<T> PipeReader<T> for Pipe<T> {
    fn read(&self) -> Option<T> {
        unsafe {
            let mut head = self.reader_head.load(Ordering::Acquire);
            if head.is_null() {
                head = self.writer_head.swap(ptr::null_mut(), Ordering::AcqRel);
                if head.is_null() {
                    return None;
                }
            }

            self.reader_head
                .store(*(*head).next.get(), Ordering::Release);

            Some((*head).value.take().unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_thread() {
        let pipe = Pipe::<u32>::new();
        pipe.write(2);
        pipe.write(1);
        pipe.write(5);
        assert_eq!(2, pipe.read().unwrap());
        assert_eq!(1, pipe.read().unwrap());
        assert_eq!(5, pipe.read().unwrap());
    }

    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_1writer_1reader() {
        const DATA_COUNT: u32 = 1000;
        let pipe = Arc::new(Pipe::<u32>::new());

        let writer_pipe = pipe.clone();
        thread::spawn(move || {
            for i in 0..DATA_COUNT {
                writer_pipe.write(i);
            }
        });

        let reader_pipe = pipe.clone();
        thread::spawn(move || {
            let mut i: u32 = 0;
            while i < DATA_COUNT {
                if let Some(val) = reader_pipe.read() {
                    assert_eq!(i, val);
                    i += 1;
                }
            }
        });
    }
}
