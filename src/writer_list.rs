
use core::ptr::Shared;
use node::{Link, Node};

pub struct Rawlink<T> {
    p: Option<Shared<T>>,
}

impl<T> Copy for Rawlink<T> {}
unsafe impl<T: Send> Send for Rawlink<T> {}
unsafe impl<T: Sync> Sync for Rawlink<T> {}

impl<T> Rawlink<T> {
    fn none() -> Rawlink<T> {
        Rawlink { p: None }
    }

    /// Like Option::Some for Rawlink
    fn some(n: &mut T) -> Rawlink<T> {
        unsafe { Rawlink { p: Some(Shared::new(n)) } }
    }

    unsafe fn resolve_mut<'a>(&mut self) -> Option<&'a mut T> {
        self.p.map(|p| &mut **p)
    }
}

impl<'a, T> From<&'a mut Link<T>> for Rawlink<Node<T>> {
    fn from(node: &'a mut Link<T>) -> Self {
        match node.as_mut() {
            None => Rawlink::none(),
            Some(ptr) => Rawlink::some(ptr),
        }
    }
}

impl<T> Clone for Rawlink<T> {
    #[inline]
    fn clone(&self) -> Rawlink<T> {
        Rawlink { p: self.p }
    }
}

pub struct WriterList<T> {
    pub head: Link<T>,
    pub tail: Rawlink<Node<T>>,
    length: usize,
}

impl<T> WriterList<T> {
    #[inline]
    pub fn new() -> WriterList<T> {
        WriterList {
            head: None,
            tail: Rawlink::none(),
            length: 0,
        }
    }
}

impl<T> Drop for WriterList<T> {
    fn drop(&mut self) {
        while let Some(mut head_) = self.head.take() {
            self.head = head_.next.take();
        }
        self.length = 0;
        self.tail = Rawlink::none();
    }
}

impl<T> WriterList<T> {
    // push to the end of the list
    #[inline]
    pub fn push_back(&mut self, elt: T) {
        let new_tail = Box::new(Node::new(elt));

        match unsafe { self.tail.resolve_mut() } {
            None => return self.push_front_node(new_tail),
            Some(tail) => {
                tail.set_next(new_tail);
                self.tail = Rawlink::from(&mut tail.next);
            }
        }
        self.length += 1;
    }

    #[inline]
    fn push_front_node(&mut self, mut new_head: Box<Node<T>>) {
        match self.head {
            None => {
                self.head = Some(new_head);
                self.tail = Rawlink::from(&mut self.head);
            }
            Some(ref mut head) => {
                head.next = Some(new_head);
            }
        }
        self.length += 1;
    }
}

#[cfg(test)]
mod tests {

}
