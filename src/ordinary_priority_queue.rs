use priority_queue::PriorityQueue;
use std::hash::Hash;

use crate::my_priority_queue::AbstractPriorityQueue;

/// the ordinary priority queue
/// of course implements the trait `AbstractPriorityQueue`
/// by redirecting to the appropriate methods
impl<T, P> AbstractPriorityQueue<T, P> for PriorityQueue<T, P>
where
    T: Eq + Hash,
    P: Ord + Clone,
{
    fn empty_copy(&self) -> Self {
        Self::with_capacity(self.my_len())
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
    fn nothing_in_out_iter() {
        use crate::Reordered;
        use priority_queue::PriorityQueue;
        let q = PriorityQueue::<u8, u8>::new();
        let it = [].into_iter();
        let mut it = Reordered::<_, _, _, _>::new(it, q, 8);
        assert!(it.next().is_none());
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
                .map(|z| (z, max_num - 1 - z))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn rev_order_inout() {
        use crate::{Reorderable, Reordered};
        use priority_queue::PriorityQueue;

        let q = PriorityQueue::<u8, u8>::new();
        let max_num = 7;

        let it = std::iter::zip(0..=max_num, 20..=max_num + 20).map(Ok);
        let it = Reordered::new(it, q, 8);
        #[allow(clippy::cast_possible_truncation)]
        for (idx, (a, b)) in it.enumerate() {
            assert_eq!(a + 20, b);
            assert_eq!(max_num - (idx as u8), a);
        }

        let q = PriorityQueue::<u8, u8>::new();
        let max_num = 40;

        let it = std::iter::zip(0..=max_num, 20..=max_num + 20).map(Ok);
        let it = Reordered::new(it, q, 4);
        let expected_as = (3..=max_num).chain([2, 1, 0]);
        for ((a, b), expected_a) in it.zip(expected_as) {
            assert_eq!(a + 20, b);
            assert_eq!(expected_a, a);
        }

        let expected_as = (6..=max_num).chain([5, 4, 3, 2, 1, 0]);
        let it = std::iter::zip(0..=max_num, 20..=max_num + 20).map(Ok);
        let q = PriorityQueue::<u8, u8>::new();
        for ((a, b), expected_a) in it.reorder(q, 7).zip(expected_as) {
            assert_eq!(a + 20, b);
            assert_eq!(expected_a, a);
        }
    }
}
