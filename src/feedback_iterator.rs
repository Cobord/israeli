use std::iter::Chain;

use crate::{AbstractPriorityQueue, Reordered};

pub enum FeedbackResult<Finished, T, P>
where
    P: Ord,
{
    FinishedOnly(Finished),
    FinishedAndFeedOne(Finished, (T, P)),
    FinishedAndFeedMany(Finished, (Vec<T>, P)),
    JustFeedOne(T, P),
    JustFeedMany(Vec<T>, P),
}

pub struct Feedback<T, P, I, Q, F, Finished>
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
    F: FnMut(T, P) -> FeedbackResult<Finished, T, P>,
{
    reordered_iterator: Reordered<T, P, I, Q>,
    processor: F,
}

impl<T, P, I, Q, F, Finished> Feedback<T, P, I, Q, F, Finished>
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
    F: FnMut(T, P) -> FeedbackResult<Finished, T, P>,
{
    pub fn new(to_feed_in: I, current_queue: Q, queue_capacity: usize, processor: F) -> Self {
        Self {
            reordered_iterator: Reordered::new(to_feed_in, current_queue, queue_capacity),
            processor,
        }
    }

    pub fn flush(&mut self) -> impl Iterator<Item = (T, P)> {
        self.reordered_iterator.flush()
    }

    pub fn chain_more<I2>(self, more_stuff: I2) -> Feedback<T, P, Chain<I, I2>, Q, F, Finished>
    where
        I2: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    {
        Feedback::<T, P, Chain<I, I2>, Q, F, Finished> {
            reordered_iterator: self.reordered_iterator.chain_more(more_stuff),
            processor: self.processor,
        }
    }

    pub fn enqueue_now(&mut self, something_now: Result<(T, P), (Vec<T>, P)>) {
        self.reordered_iterator.enqueue_now(something_now);
    }
}

impl<T, P, I, Q, F, Finished> Iterator for Feedback<T, P, I, Q, F, Finished>
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
    F: FnMut(T, P) -> FeedbackResult<Finished, T, P>,
{
    type Item = Finished;

    fn next(&mut self) -> Option<Self::Item> {
        let to_process = self.reordered_iterator.next();
        if let Some(to_process) = to_process {
            let processed = (self.processor)(to_process.0, to_process.1);
            match processed {
                FeedbackResult::FinishedOnly(f) => Some(f),
                FeedbackResult::FinishedAndFeedOne(f, (this_push, this_priority)) => {
                    self.enqueue_now(Ok((this_push, this_priority)));
                    Some(f)
                }
                FeedbackResult::FinishedAndFeedMany(f, (this_push, this_priority)) => {
                    self.enqueue_now(Err((this_push, this_priority)));
                    Some(f)
                }
                FeedbackResult::JustFeedOne(this_push, this_priority) => {
                    self.enqueue_now(Ok((this_push, this_priority)));
                    self.next()
                }
                FeedbackResult::JustFeedMany(this_push, this_priority) => {
                    self.enqueue_now(Err((this_push, this_priority)));
                    self.next()
                }
            }
        } else {
            None
        }
    }
}
