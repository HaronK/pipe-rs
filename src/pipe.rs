
use core::ptr;
use core::cell::UnsafeCell;

use std::sync::atomic::{AtomicPtr, Ordering};

struct Node<T> {
    next:  UnsafeCell<*mut Node<T>>,
    value: Option<T>,
}

pub trait PipeWriter<T> {
    fn write(&mut self, elt: T);
}

pub trait PipeReader<T> {
    fn read(&mut self) -> Option<T>;
}

pub struct Pipe<T> {
    writer_head:     AtomicPtr<Node<T>>,
    writer_tail:     UnsafeCell<*mut Node<T>>,
    writer_finished: bool,

    reader_head:     AtomicPtr<Node<T>>,
}

unsafe impl<T: Send> Send for Pipe<T> { }
unsafe impl<T: Send> Sync for Pipe<T> { }

impl<T> Node<T> {
    #[inline]
    unsafe fn new(v: T) -> *mut Node<T> {
        Box::into_raw(Box::new(Node {
            next:  UnsafeCell::new(ptr::null_mut()),
            value: Some(v),
        }))
    }
}

impl<T> Pipe<T> {
    #[inline]
    pub fn new() -> Pipe<T> {
        Pipe {
            writer_head:     AtomicPtr::new(ptr::null_mut()),
            writer_tail:     UnsafeCell::new(ptr::null_mut()),
            writer_finished: false,
            reader_head:     AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl<T> Drop for Pipe<T> {
    fn drop(&mut self) {
        // TODO: implement
    }
}

impl<T> PipeWriter<T> for Pipe<T> {
    fn write(&mut self, elt: T) {
        assert!(!self.writer_finished);

        unsafe {
            let node = Node::new(elt);

            let mut head = self.writer_head.swap(ptr::null_mut(), Ordering::AcqRel);
            if head.is_null() {
                head = node;
            }
            else {
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
    fn read(&mut self) -> Option<T> {
        unsafe {
            let mut head = self.reader_head.load(Ordering::Acquire);
            if head.is_null() {
                head = self.writer_head.swap(ptr::null_mut(), Ordering::AcqRel);
                if head.is_null() {
                    return None;
                }
            }
    
            self.reader_head.store(*(*head).next.get(), Ordering::Release);

            Some((*head).value.take().unwrap())
        }
    }
}

#[cfg(test)]
mod tests {

}
