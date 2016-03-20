
//use std::sync::Mutex;
//
//pub struct Guard<T: Drop> {
//    obj: T,
//    mutex: Mutex,
//}
//
//impl<T> Guard<T> {
//    #[inline]
//    pub fn new(obj: T) -> Guard<T> {
//        let res = Guard {
//            obj: obj,
//            mutex: Mutex::new(obj),
//        }
//        res.mutex.lock().unwrap();
//    }
//}
//
//impl<T> Drop for Guard<T> {
//    fn drop(&mut self) {
//        while let Some(mut head_) = self.head.take() {
//            self.head = head_.next.take();
//        }
//        self.length = 0;
//        self.tail = Rawlink::none();
//    }
//}
