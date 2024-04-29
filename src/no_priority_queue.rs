use std::collections::VecDeque;

use crate::my_priority_queue::AbstractPriorityQueue;

#[repr(transparent)]
struct NoPriorityQueue<T, P>(VecDeque<(T, P)>);

impl<T, P> NoPriorityQueue<T, P>
where
    P: Ord + Default,
{
    #[allow(dead_code)]
    fn new() -> Self {
        Self(VecDeque::new())
    }
}

impl<T, P> AbstractPriorityQueue<T, P> for NoPriorityQueue<T, P>
where
    P: Ord + Default,
{
    fn empty_copy(&self) -> Self {
        todo!()
    }

    fn my_peek(&self) -> Option<(&T, &P)> {
        #[allow(clippy::map_identity)]
        self.0.front().map(|(z, w)| (z, w))
    }

    fn my_enqueue(&mut self, new_obj: T, _new_obj_priority: P) {
        self.0.push_back((new_obj, P::default()))
    }

    fn enqueue_batch(&mut self, new_batch: impl IntoIterator<Item = T>, _new_batch_priority: P) {
        self.0
            .extend(new_batch.into_iter().map(|z| (z, P::default())))
    }

    fn my_dequeue(&mut self) -> Option<(T, P)> {
        self.0.pop_front()
    }

    fn dequeue_batch(&mut self, around_how_many: usize, hard_limit: usize) -> Vec<(T, P)> {
        assert!(hard_limit >= around_how_many);
        let around_how_many = std::cmp::min(around_how_many, self.0.len());
        self.0.drain(0..around_how_many).collect()
    }

    fn my_len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

mod test {

    #[test]
    fn nothing_in_out() {
        use super::NoPriorityQueue;
        use crate::my_priority_queue::AbstractPriorityQueue;
        let mut q = NoPriorityQueue::<u8, u8>::new();
        assert_eq!(q.my_len(), 0);
        assert!(q.is_empty());
        assert!(q.my_dequeue().is_none());
        assert!(q.dequeue_batch(10, 10).is_empty());
    }
}
