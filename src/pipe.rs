
use std::mem;
use std::sync::atomic::Ordering;
use writer_list::WriterList;
use reader_list::ReaderList;

pub trait PipeWriter<T> {
    fn write(&mut self, elt: T);
}

pub trait PipeReader<T> {
    fn read(&mut self) -> Option<T>;
}

pub struct Pipe<T> {
    writer: WriterList<T>,
    reader: ReaderList<T>,
    writer_finished: bool,
}

impl<T> Pipe<T> {
    #[inline]
    pub fn new() -> Pipe<T> {
        Pipe {
            writer: WriterList::new(),
            reader: ReaderList::new(),
            writer_finished: false,
        }
    }
}

impl<T> PipeWriter<T> for Pipe<T> {
    fn write(&mut self, elt: T) {
        assert!(!self.writer_finished);

        self.writer.push_back(elt);

        if self.reader.set_head_if_empty(mem::replace(&mut self.writer.head, None)) {
            self.writer.head = None;
        }
    }
}

impl<T> PipeReader<T> for Pipe<T> {
    fn read(&mut self) -> Option<T> {
        let value = self.reader.pop_front();
        if value.is_none() {
            if self.writer_finished && self.writer.head.is_some() {
                self.reader.set_head(mem::replace(&mut self.writer.head, None));
                value = self.writer.head;
                self.writer.head = None;
            } else {
                return None;
            }
        }

        let data = value.unwrap();
        self.reader.set_head(data.next);
        // data.next = None;

        Some((*data).value)
    }
}

#[cfg(test)]
mod tests {

}
