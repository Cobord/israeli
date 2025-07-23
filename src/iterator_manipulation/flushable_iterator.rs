pub trait FlushableIterator: Iterator {
    /// This iterator has some `AbstractPriorityQueue` `Q`
    /// temporary storage which is where items are possibly
    /// reprioritized. This means we can clear that part of the
    /// items and leave that `Q` in it's empty state for
    /// whatever is left from the inner `Iterator`
    fn flush(&mut self) -> impl Iterator<Item = Self::Item>;
}
