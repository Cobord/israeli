pub trait AbstractPriorityQueue<T, P: Ord> {
    fn empty_copy(&self) -> Self;

    fn my_peek(&self) -> Option<(&T, &P)>;

    fn my_enqueue(&mut self, new_obj: T, new_obj_priority: P);

    /*
    enqueueing many items all with the same priority, might have a faster implementation than
    just doing them one by one
    */
    fn enqueue_batch(&mut self, new_batch: impl IntoIterator<Item = T>, new_batch_priority: P);

    fn my_dequeue(&mut self) -> Option<(T, P)>;

    /*
    if there are fewer than around_how_many gives all of the items in the queue
    if there are more than that, give some number of items
         that is at least around_how_many but less than or equal to than the hard limit
         where how much more depends on the specific implementer and the specific items involved
    */
    fn dequeue_batch(&mut self, around_how_many: usize, hard_limit: usize) -> Vec<(T, P)>;

    fn all_items_iter(mut self) -> impl Iterator<Item = T>
    where
        Self: Sized,
    {
        self.dequeue_batch(self.my_len(), self.my_len())
            .into_iter()
            .map(|z| z.0)
    }

    fn my_len(&self) -> usize;
    fn is_empty(&self) -> bool;

    fn drain_all(&mut self) -> Vec<(T, P)> {
        self.dequeue_batch(self.my_len(), self.my_len())
    }
}
