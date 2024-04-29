use priority_queue::PriorityQueue;
use std::hash::Hash;

use crate::my_priority_queue::AbstractPriorityQueue;

impl<T, P> AbstractPriorityQueue<T, P> for PriorityQueue<T, P>
where
    T: Eq + Hash,
    P: Ord + Clone,
{
    fn empty_copy(&self) -> Self {
        todo!()
    }

    fn my_peek(&self) -> Option<(&T, &P)> {
        self.peek()
    }

    fn my_enqueue(&mut self, new_obj: T, new_obj_priority: P) {
        self.push(new_obj, new_obj_priority);
    }

    fn my_dequeue(&mut self) -> Option<(T, P)> {
        self.pop()
    }

    fn all_items_iter(self) -> impl Iterator<Item = T> {
        self.into_sorted_vec().into_iter()
    }

    fn my_len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn dequeue_batch(&mut self, around_how_many: usize, hard_limit: usize) -> Vec<(T, P)> {
        assert!(hard_limit >= around_how_many);
        let mut to_return = Vec::with_capacity(around_how_many);
        for _ in 0..around_how_many {
            let put_in = self.my_dequeue();
            if let Some(real_put_in) = put_in {
                to_return.push(real_put_in);
            } else {
                break;
            }
        }
        to_return
    }

    fn enqueue_batch(&mut self, new_batch: impl IntoIterator<Item = T>, new_batch_priority: P) {
        for b in new_batch {
            self.my_enqueue(b, new_batch_priority.clone());
        }
    }
}

mod test {

    #[test]
    fn nothing_in_out() {
        use crate::my_priority_queue::AbstractPriorityQueue;
        use priority_queue::PriorityQueue;
        let mut q = PriorityQueue::<u8, u8>::new();
        assert_eq!(q.my_len(), 0);
        assert!(q.is_empty());
        assert!(q.my_dequeue().is_none());
        assert!(q.dequeue_batch(10, 10).is_empty());
    }

    #[test]
    fn same_order_inout() {
        use crate::my_priority_queue::AbstractPriorityQueue;
        use priority_queue::PriorityQueue;
        let mut q = PriorityQueue::<u8, u8>::new();
        let max_num = 7;
        for (a, b) in std::iter::zip(0..max_num, (0..max_num).rev()) {
            q.my_enqueue(a, b);
        }
        assert_eq!(q.my_len(), max_num.into());
        assert!(!q.is_empty());
        let removed = q.my_dequeue();
        assert_eq!(removed, Some((0, max_num - 1)));
        assert_eq!(
            q.dequeue_batch(10, 10),
            (1..max_num)
                .into_iter()
                .map(|z| (z, max_num - 1 - z))
                .collect::<Vec<_>>()
        );
    }
}
