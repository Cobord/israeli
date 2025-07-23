use std::iter::Chain;

use crate::{AbstractPriorityQueue, FlushableIterator, Reordered};

/// From the output of the inner iterator we can either just output it
/// or put some stuff back in the `AbstractPriorityQueue<T,P>`
/// or both.
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

/// Hold an iterator `reordered_iterator`
/// that uses the `AbstractPriorityQueue<T,P>` `Q`
/// in order to reprioritize how items come out.
/// As we take those items out, we
/// post-process with `F` which returns a `FeedbackResult<Finished,T,P>`.
/// Some of them are ready to output in final form as `Finished`.
/// Some of them turn into things that need to go back into `Q`.
/// Some of them do both.
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
            reordered_iterator: super::Reorderable::reorder(
                to_feed_in,
                current_queue,
                queue_capacity,
            ),
            processor,
        }
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

impl<T, P, I, Q, F, Finished> FlushableIterator for Feedback<T, P, I, Q, F, Finished>
where
    I: Iterator<Item = Result<(T, P), (Vec<T>, P)>>,
    Q: AbstractPriorityQueue<T, P>,
    P: Ord,
    F: FnMut(T, P) -> FeedbackResult<Finished, T, P>,
{
    fn flush(&mut self) -> impl Iterator<Item = Self::Item> {
        let flushed_queue: Vec<_> = self.reordered_iterator.flush().collect();
        flushed_queue
            .into_iter()
            .filter_map(|to_process| {
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
                        None
                    }
                    FeedbackResult::JustFeedMany(this_push, this_priority) => {
                        self.enqueue_now(Err((this_push, this_priority)));
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
