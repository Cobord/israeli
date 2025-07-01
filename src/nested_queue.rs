use crate::my_priority_queue::AbstractPriorityQueue;
use std::{collections::HashMap, hash::Hash, marker::PhantomData};

/// the fine grained priorities can be coarse grained into this type
/// which means we have a monotone map
pub trait CoarseGrainedPriority<P> {
    fn coarse_grain(p: &P) -> Self;
    fn decrement(&mut self);
}

/// there is a way to lookup and insert items by C
/// and the associated data is of type Q
pub trait IndexInto<C, Q> {
    fn new() -> Self;
    fn get(&self, which: &C) -> Option<&Q>;
    fn get_mut(&mut self, which: &C) -> Option<&mut Q>;
    fn insert(&mut self, which: C, value: Q) -> Option<Q>;
    fn remove(&mut self, which: &C) -> Option<Q>;
    fn values<'a>(&'a self) -> impl Iterator<Item = &'a Q>
    where
        Q: 'a;
}

/// redirect to the corresponding methods in `HashMap`
impl<C, Q, S: ::std::hash::BuildHasher + Default> IndexInto<C, Q> for HashMap<C, Q, S>
where
    C: Hash + Eq,
{
    fn new() -> Self {
        HashMap::default()
    }

    fn get(&self, which: &C) -> Option<&Q> {
        self.get(which)
    }

    fn get_mut(&mut self, which: &C) -> Option<&mut Q> {
        self.get_mut(which)
    }

    fn insert(&mut self, which: C, value: Q) -> Option<Q> {
        self.insert(which, value)
    }

    fn remove(&mut self, which: &C) -> Option<Q> {
        self.remove(which)
    }

    fn values<'a>(&'a self) -> impl Iterator<Item = &'a Q>
    where
        Q: 'a,
    {
        self.values()
    }
}

/// redirect to element access and setting
impl<Q> IndexInto<usize, Q> for Vec<Option<Q>> {
    fn new() -> Self {
        Vec::new()
    }

    fn get(&self, which: &usize) -> Option<&Q> {
        if *which < self.len() {
            if let Some(a) = &self[*which] {
                Some(a)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_mut(&mut self, which: &usize) -> Option<&mut Q> {
        if *which < self.len() {
            if let Some(a) = &mut self[*which] {
                Some(a)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn insert(&mut self, which: usize, value: Q) -> Option<Q> {
        if which < self.len() {
            let mut return_value = Some(value);
            std::mem::swap(&mut return_value, &mut self[which]);
            return_value
        } else {
            self.extend((0..which - self.len()).map(|_| None));
            self.insert(which, Some(value));
            None
        }
    }

    fn remove(&mut self, which: &usize) -> Option<Q> {
        if *which < self.len() {
            let to_return = std::mem::take(&mut self[*which]);
            #[allow(clippy::redundant_closure_for_method_calls)]
            if self[*which..].iter().all(|z| z.is_none()) {
                self.drain(*which..);
            }
            to_return
        } else {
            None
        }
    }

    fn values<'a>(&'a self) -> impl Iterator<Item = &'a Q>
    where
        Q: 'a,
    {
        self.iter()
            .filter_map(|z| if let Some(a) = z { Some(a) } else { None })
    }
}

/// the items of type T and priority P being stored are being stored in one of potentially
/// several Q's which are all the same kind of `AbstractPriorityQueue` for such items and priorities
/// but they are divided up by the coarsed grained priority
/// so each one of those are smaller and only storing items with priorities with the same
/// coarse grained priority
pub struct BucketQueue<T, P, C, Q, Storer>
where
    C: CoarseGrainedPriority<P> + Hash + Ord + Clone,
    P: Ord,
    Q: AbstractPriorityQueue<T, P>,
    Storer: IndexInto<C, Q>,
{
    my_buckets: Storer,
    upper_bound_occupied_bucket: C,
    lower_bound_occupied_bucket: C,
    junk: PhantomData<T>,
    junk2: PhantomData<P>,
    bucket_template: Q,
}

impl<T, P, C, Q, Storer> Default for BucketQueue<T, P, C, Q, Storer>
where
    C: CoarseGrainedPriority<P> + Hash + Ord + Clone + Default,
    P: Ord,
    Q: AbstractPriorityQueue<T, P> + Default,
    Storer: IndexInto<C, Q>,
{
    fn default() -> Self {
        Self {
            my_buckets: Storer::new(),
            upper_bound_occupied_bucket: C::default(),
            lower_bound_occupied_bucket: C::default(),
            junk: PhantomData,
            junk2: PhantomData,
            bucket_template: Q::default(),
        }
    }
}

impl<T, P, C, Q, Storer> BucketQueue<T, P, C, Q, Storer>
where
    C: CoarseGrainedPriority<P> + Hash + Ord + Clone,
    P: Ord,
    Q: AbstractPriorityQueue<T, P>,
    Storer: IndexInto<C, Q>,
{
    /// pass a lower and upper bound for the coarse grained priorities you are expecting to see
    /// these don't have to be accurate, but if you pass them accurately it is better
    /// you also pass an example of Q that is to be used for constructing new buckets
    pub fn new(lower_bound_occupied_bucket: C, upper_bound_occupied_bucket: C, dummy: &Q) -> Self {
        Self {
            my_buckets: Storer::new(),
            upper_bound_occupied_bucket,
            lower_bound_occupied_bucket,
            junk: PhantomData,
            junk2: PhantomData,
            bucket_template: dummy.empty_copy(),
        }
    }
}

impl<T, P, C, Q, Storer> AbstractPriorityQueue<T, P> for BucketQueue<T, P, C, Q, Storer>
where
    C: CoarseGrainedPriority<P> + Hash + Ord + Clone,
    P: Ord,
    Q: AbstractPriorityQueue<T, P>,
    Storer: IndexInto<C, Q>,
{
    fn empty_copy(&self) -> Self {
        Self {
            my_buckets: Storer::new(),
            upper_bound_occupied_bucket: self.upper_bound_occupied_bucket.clone(),
            lower_bound_occupied_bucket: self.lower_bound_occupied_bucket.clone(),
            junk: self.junk,
            junk2: self.junk2,
            bucket_template: self.bucket_template.empty_copy(),
        }
    }

    fn my_peek(&self) -> Option<(&T, &P)> {
        let mut looking_in_bucket = self.upper_bound_occupied_bucket.clone();
        while looking_in_bucket >= self.lower_bound_occupied_bucket {
            if let Some(cur_bucket) = self.my_buckets.get(&looking_in_bucket) {
                #[allow(clippy::single_match)]
                match cur_bucket.my_peek() {
                    z @ Some(_) => {
                        return z;
                    }
                    None => {}
                }
            }
            looking_in_bucket.decrement();
        }
        None
    }

    fn my_enqueue(&mut self, new_obj: T, new_obj_priority: P) {
        let which_bucket = C::coarse_grain(&new_obj_priority);
        self.lower_bound_occupied_bucket = std::cmp::min(
            self.lower_bound_occupied_bucket.clone(),
            which_bucket.clone(),
        );
        self.upper_bound_occupied_bucket = std::cmp::max(
            self.upper_bound_occupied_bucket.clone(),
            which_bucket.clone(),
        );
        if let Some(cur_bucket) = self.my_buckets.get_mut(&which_bucket) {
            cur_bucket.my_enqueue(new_obj, new_obj_priority);
        } else {
            let mut new_bucket = self.bucket_template.empty_copy();
            new_bucket.my_enqueue(new_obj, new_obj_priority);
            self.my_buckets.insert(which_bucket, new_bucket);
        }
    }

    fn enqueue_batch(&mut self, new_batch: impl IntoIterator<Item = T>, new_batch_priority: P) {
        let which_bucket = C::coarse_grain(&new_batch_priority);
        self.lower_bound_occupied_bucket = std::cmp::min(
            self.lower_bound_occupied_bucket.clone(),
            which_bucket.clone(),
        );
        self.upper_bound_occupied_bucket = std::cmp::max(
            self.upper_bound_occupied_bucket.clone(),
            which_bucket.clone(),
        );
        if let Some(cur_bucket) = self.my_buckets.get_mut(&which_bucket) {
            cur_bucket.enqueue_batch(new_batch, new_batch_priority);
        } else {
            let mut new_bucket = self.bucket_template.empty_copy();
            new_bucket.enqueue_batch(new_batch, new_batch_priority);
            self.my_buckets.insert(which_bucket, new_bucket);
        }
    }

    fn my_dequeue(&mut self) -> Option<(T, P)> {
        while self.upper_bound_occupied_bucket >= self.lower_bound_occupied_bucket {
            if let Some(cur_bucket) = self.my_buckets.get_mut(&self.upper_bound_occupied_bucket) {
                let ret_item = cur_bucket.my_dequeue();
                if cur_bucket.is_empty() {
                    let _is_cur_bucket = self.my_buckets.remove(&self.upper_bound_occupied_bucket);
                    self.upper_bound_occupied_bucket.decrement();
                }
                return ret_item;
            }
            self.upper_bound_occupied_bucket.decrement();
        }
        None
    }

    fn dequeue_batch(&mut self, around_how_many: usize, hard_limit: usize) -> Vec<(T, P)> {
        let mut to_return = Vec::with_capacity(around_how_many);
        let mut to_return_len = 0;
        while self.upper_bound_occupied_bucket >= self.lower_bound_occupied_bucket {
            if let Some(cur_bucket) = self.my_buckets.get_mut(&self.upper_bound_occupied_bucket) {
                let this_batch = cur_bucket
                    .dequeue_batch(around_how_many - to_return_len, hard_limit - to_return_len);
                to_return_len += this_batch.len();
                to_return.extend(this_batch);
                if cur_bucket.is_empty() {
                    let _is_cur_bucket = self.my_buckets.remove(&self.upper_bound_occupied_bucket);
                }
                if to_return_len >= around_how_many {
                    break;
                }
            }
            self.upper_bound_occupied_bucket.decrement();
        }
        to_return
    }

    fn my_len(&self) -> usize {
        let mut total_len = 0;
        for v in self.my_buckets.values() {
            total_len += v.my_len();
        }
        total_len
    }

    fn is_empty(&self) -> bool {
        for v in self.my_buckets.values() {
            if !v.is_empty() {
                return false;
            }
        }
        true
    }
}

mod test {}
