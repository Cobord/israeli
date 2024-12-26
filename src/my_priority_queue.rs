pub trait AbstractPriorityQueue<T, P: Ord> {
    /// a version just like self
    /// but without any actual items
    #[must_use]
    fn empty_copy(&self) -> Self;

    /// what would `my_dequeue` give
    fn my_peek(&self) -> Option<(&T, &P)>;

    /// insert an item with specified priority
    fn my_enqueue(&mut self, new_obj: T, new_obj_priority: P);

    /// enqueueing many items all with the same priority, might have a faster implementation than
    /// just doing them one by one
    fn enqueue_batch(&mut self, new_batch: impl IntoIterator<Item = T>, new_batch_priority: P);

    /// if nonempty, one item comes out
    /// the order is dependent on the specific implementation
    /// and how it handles priorities
    fn my_dequeue(&mut self) -> Option<(T, P)>;

    /// if there are fewer than `around_how_many` gives all of the items in the queue
    /// if there are more than that, give some number of items
    ///     that is at least `around_how_many` but less than or equal to than the `hard limit`
    ///     where how much more depends on the specific implementer and the specific items involved
    fn dequeue_batch(&mut self, around_how_many: usize, hard_limit: usize) -> Vec<(T, P)>;

    /// with no more items being enqueue'd
    /// we can just dequeue them all and provide an iterator
    /// over all the items ignoring their priorities
    fn all_items_iter(mut self) -> impl Iterator<Item = T>
    where
        Self: Sized,
    {
        self.dequeue_batch(self.my_len(), self.my_len())
            .into_iter()
            .map(|z| z.0)
    }

    /// how many items are present
    fn my_len(&self) -> usize;

    /// is the queue empty
    fn is_empty(&self) -> bool;

    /// dequeue them all
    /// but after the mutable reference issues resolved
    /// more items can still be enqueue'd
    /// unlike ``all_items_iter`` which closed this off
    fn drain_all(&mut self) -> Vec<(T, P)> {
        self.dequeue_batch(self.my_len(), self.my_len())
    }
}
