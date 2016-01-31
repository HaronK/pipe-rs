
use std::sync::atomic::{AtomicPtr, Ordering};
use node::Link;

pub struct ReaderList<T> {
    head: AtomicPtr<Link<T>>,
    length: usize,
}

impl<T> ReaderList<T> {
    #[inline]
    pub fn new() -> ReaderList<T> {
        ReaderList {
            head: AtomicPtr::new(&mut None),
            length: 0,
        }
    }
}

impl<T> Drop for ReaderList<T> {
    fn drop(&mut self) {
        let mut elem = self.get_head();
        while let Some(mut head_) = elem.take() {
            elem = head_.next.take();
        }
        self.length = 0;
        self.set_head(None);
    }
}

impl<T> ReaderList<T> {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.get_head().is_none()
    }

    #[inline]
    pub fn set_head_if_empty(&mut self, head: Link<T>) -> bool {
        match self.is_empty() {
            true => {
                self.set_head(head);
                true
            }
            false => false,
        }
    }

    #[inline]
    pub fn set_head(&mut self, head: Link<T>) {
        // self.head.store(head, Ordering::Release);
        match head {
            None => self.head.store(&mut None, Ordering::Release),
            Some(data) => self.head.store(&mut Some(data), Ordering::Release),
        }
    }

    // pop element from the top of the list
    #[inline]
    pub fn pop_front(&mut self) -> Link<T> {
        let head = self.get_head();
        match head {
            None => None,
            Some(v) => {
                self.set_head(v.next);
                head
            }
        }
    }

    #[inline]
    fn get_head(&self) -> Link<T> {
        unsafe { *self.head.load(Ordering::Acquire) }
    }
}

#[cfg(test)]
mod tests {

}
