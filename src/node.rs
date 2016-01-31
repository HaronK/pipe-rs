
use alloc::boxed::Box;

pub type Link<T> = Option<Box<Node<T>>>;

pub struct Node<T> {
    pub next: Link<T>,
    pub value: T,
}

impl<T> Node<T> {
    pub fn new(v: T) -> Node<T> {
        Node {
            value: v,
            next: None,
        }
    }

    /// Set self's next pointer.
    ///
    /// `self.next` should be `None` when you call this
    /// (otherwise a Node is probably being dropped by mistake).
    pub fn set_next(&mut self, mut next: Box<Node<T>>) {
        debug_assert!(self.next.is_none());
        self.next = Some(next);
    }
}

#[cfg(test)]
mod tests {

}
