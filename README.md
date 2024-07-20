## AbstractPriorityQueue

In order to create a uniform set of functions that all sorts of priority queues use, they are encompassed in this trait. This is even if the implementer is doing more than an ordinary PriorityQueue like the IsraeliQueue is.

- create an empty copy (this is done over new because of maintaining parts like how to combine priorities in the IsraeliQueue)
- peek at the next item and it's priority
- enqueue a new item with specified priority
- enqueue several items all with the same priority
- dequeue the next item with it's priority
- dequeue several items
    - it will attempt to get the minimum of how many are in the queue and around_how_many
    - it may overshoot and give more than that as long as it is less than the hard_limit
- an iterator over all the items that just uses the above with the lengths being everything
- query the length
- query if it is empty
- drain everything into a vector

## Israeli Queue

This isn't really an Israeli queue, because we have a shibboleth rather than iterating through to look for friends. But by using a trait with a generic we can avoid that iteration.

The items must implement Friendly<H> which gives the shibboleth for that item belonging to a certain friend group.

That way we have the regular priority queue for the shibboleths and how the shibboleths translate to nonempty lists of items.

The specification of how priorities combine when a new item joins a currently waiting friend group is variable. Often it is described as being the maximum priority of any of the friends. Sometimes it is the sum.
Instead of committing to one of these, the function to do so is part of the queue itself. It defaults to using the maximum, but you can change it.

## Nested Queues

Consider the bucket queue, each bucket stores items of the same priority. Instead of that here we have a coarse grained priority which is a monotone function of the original priorities. The individual buckets are now something that implements
AbstractPriorityQueue and the AbstractPriorityQueue operations of the nested queue use those operations on the individual buckets as appropriate

## No Priority, Ordinary Priority

### No priority

If priorities don't matter a transparent wrapper of VecDequeue implements these operations and ignores the priorities

### Ordinary Priority

PriorityQueue from the priority_queue crate implements the trait as well.

###
