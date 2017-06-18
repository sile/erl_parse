use super::parts::{Sequence, SequenceTail};

#[derive(Debug)]
pub struct SequenceIter<'a, T: 'a, D: 'a>(SequenceIterInner<'a, T, D>);
impl<'a, T: 'a, D: 'a> SequenceIter<'a, T, D> {
    pub fn new(seq: &'a Sequence<T, D>) -> Self {
        let inner = SequenceIterInner::Head(seq);
        SequenceIter(inner)
    }
}
impl<'a, T: 'a, D: 'a> Iterator for SequenceIter<'a, T, D> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Debug)]
enum SequenceIterInner<'a, T: 'a, D: 'a> {
    Head(&'a Sequence<T, D>),
    Tail(&'a SequenceTail<T, D>),
    Eos,
}
impl<'a, T: 'a, D: 'a> Iterator for SequenceIterInner<'a, T, D> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            SequenceIterInner::Head(&Sequence { ref item, ref tail }) => {
                if let Some(ref tail) = *tail {
                    *self = SequenceIterInner::Tail(tail);
                } else {
                    *self = SequenceIterInner::Eos
                }
                Some(item)
            }
            SequenceIterInner::Tail(&SequenceTail { ref item, ref tail, .. }) => {
                if let Some(ref tail) = *tail {
                    *self = SequenceIterInner::Tail(tail);
                } else {
                    *self = SequenceIterInner::Eos
                }
                Some(item)
            }
            SequenceIterInner::Eos => None,
        }
    }
}
