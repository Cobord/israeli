use crate::{AbstractPriorityQueue, FlushableIterator};

pub struct Reordered<T, P, I, Q>
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
{
    to_feed_in: I,
    current_queue: Q,
    queue_capacity: usize,
}

impl<T, P, I, Q> Reordered<T, P, I, Q>
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
{
    pub fn new(to_feed_in: I, current_queue: Q, queue_capacity: usize) -> Self {
        Self {
            to_feed_in,
            current_queue,
            queue_capacity,
        }
    }

    pub fn chain_more<I2>(self, more_stuff: I2) -> Reordered<T, P, std::iter::Chain<I, I2>, Q>
    where
        I2: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    {
        Reordered::<T, P, std::iter::Chain<I, I2>, Q> {
            to_feed_in: self.to_feed_in.chain(more_stuff),
            current_queue: self.current_queue,
            queue_capacity: self.queue_capacity,
        }
    }

    pub fn enqueue_now(&mut self, something_now: Result<(T, P), (Vec<T>, P)>) {
        match something_now {
            Ok((cur_t, cur_p)) => {
                self.current_queue.my_enqueue(cur_t, cur_p);
            }
            Err((cur_ts, cur_p)) => {
                self.current_queue.enqueue_batch(cur_ts, cur_p);
            }
        }
    }
}

pub trait Reorderable<T, P, Q>: Iterator<Item = Result<(T, P), (Vec<T>, P)>> + Sized
where
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
{
    fn reorder(self, current_queue: Q, queue_capacity: usize) -> Reordered<T, P, Self, Q> {
        Reordered::new(self, current_queue, queue_capacity)
    }
}

impl<T, P, I, Q> Reorderable<T, P, Q> for I
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
{
}

impl<T, P, I, Q> Iterator for Reordered<T, P, I, Q>
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
{
    type Item = (T, P);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_queue.my_len() < self.queue_capacity {
            if let Some(res) = self.to_feed_in.next() {
                match res {
                    Ok((new_obj, new_obj_priority)) => {
                        self.current_queue.my_enqueue(new_obj, new_obj_priority);
                    }
                    Err((new_obj, new_obj_priority)) => {
                        self.current_queue.enqueue_batch(new_obj, new_obj_priority);
                    }
                }
            } else {
                break;
            }
        }
        if let Some((my_obj, my_obj_priority)) = self.current_queue.my_dequeue() {
            Some((my_obj, my_obj_priority))
        } else {
            None
        }
    }
}

impl<T, P, I, Q> FlushableIterator for Reordered<T, P, I, Q>
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
{
    fn flush(&mut self) -> impl Iterator<Item = (T, P)> {
        let mut replace_queue = self.current_queue.empty_copy();
        core::mem::swap(&mut replace_queue, &mut self.current_queue);
        replace_queue.drain_all().into_iter()
    }
}
