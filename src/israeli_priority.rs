use nonempty::NonEmpty;
use priority_queue::PriorityQueue;
use std::{cmp::Ordering, collections::HashMap, hash::Hash};

use crate::my_priority_queue::AbstractPriorityQueue;

/// get a shibooleth that we can compare with equality
/// to judge if two items in the queue are friends or not
pub trait Friendly<H: Hash + Eq> {
    /// use the shibboleths to determine if they are friends
    fn is_friend(&self, other: &Self) -> bool {
        self.friendship_shibboleth() == other.friendship_shibboleth()
    }

    /// get the shibboleth
    fn friendship_shibboleth(&self) -> H;
}

/// are all items in that nonempty collection friends of each other
fn are_all_friends<T: Friendly<H>, H: Hash + Eq>(new_batch: &NonEmpty<T>) -> (bool, H) {
    let the_shibboleth = new_batch.head.friendship_shibboleth();
    let all_friends = new_batch
        .tail
        .iter()
        .all(|r| r.friendship_shibboleth() == the_shibboleth);
    (all_friends, the_shibboleth)
}

/// This isn't really an Israeli queue,
/// because we have a shibboleth rather than iterating through to look for friends.
/// But by using a trait with a generic we can avoid that iteration.
/// That way we have the regular priority queue for the shibboleths
/// and how the shibboleths translate to nonempty lists of items.
pub struct IsraeliPriority<T, P, H>
where
    T: Friendly<H>,
    P: Ord + Clone,
    H: Eq + Hash + Clone,
{
    underlying: PriorityQueue<H, P>,
    current_friend_group: Option<(NonEmpty<T>, P)>,
    shibboleth_to_friends: HashMap<H, NonEmpty<T>>,
    waiting_len: usize,
    priority_combiner: fn(&P, &P) -> (bool, P),
}

impl<T, P, H> Default for IsraeliPriority<T, P, H>
where
    T: Friendly<H>,
    P: Ord + Clone,
    H: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::with_capacity(8)
    }
}

impl<T, P, H> IsraeliPriority<T, P, H>
where
    T: Friendly<H>,
    P: Ord + Clone,
    H: Eq + Hash + Clone,
{
    /// setup with capacity for specified number of distinct friend groups
    /// the way priorities of the portion already in line and a new friend joining
    /// that friend group is the default which just picks the bigger of the two
    /// that way a high priority member joining a group in line, can boost
    /// their priority and move that entire group up even further
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let default_combiner = |p1: &P, p2: &P| {
            if p1.cmp(p2) == Ordering::Less {
                (true, p2.clone())
            } else {
                (false, p1.clone())
            }
        };
        Self {
            underlying: PriorityQueue::with_capacity(capacity),
            current_friend_group: None,
            shibboleth_to_friends: HashMap::with_capacity(capacity >> 3),
            waiting_len: 0,
            priority_combiner: default_combiner,
        }
    }

    /// provide a different way for priorities to combine
    /// when a portion is already in line and a new friend joins
    /// that friend group
    pub fn change_combiner(&mut self, new_combiner: fn(&P, &P) -> (bool, P)) {
        self.priority_combiner = new_combiner;
    }

    /// either there is a batch of friends currently in the process of going in
    /// get them all and their shared priority
    /// otherwise dequeue only one element and that can bring their friends
    /// as well through the functionality of `my_dequeue`
    fn israeli_dequeue_batch(&mut self) -> Option<(NonEmpty<T>, P)> {
        if let Some(current_batch) = self.current_friend_group.take() {
            Some((current_batch.0, current_batch.1))
        } else {
            let first_in_next_batch = self.my_dequeue();
            if let Some((rest_of_batch, priority)) = self.current_friend_group.take() {
                let real_first =
                    first_in_next_batch.expect("If there was a rest, then there would be a first");
                let mut ret_val = NonEmpty::singleton(real_first.0);
                ret_val.extend(rest_of_batch);
                Some((ret_val, priority))
            } else {
                first_in_next_batch.map(|(z, w)| (NonEmpty::singleton(z), w))
            }
        }
    }
}

impl<T, P, H> AbstractPriorityQueue<T, P> for IsraeliPriority<T, P, H>
where
    T: Friendly<H>,
    P: Ord + Clone,
    H: Hash + Eq + Clone,
{
    #[must_use]
    fn empty_copy(&self) -> Self {
        let mut to_return = Self::with_capacity(self.my_len());
        to_return.change_combiner(self.priority_combiner);
        to_return
    }

    fn my_peek(&self) -> Option<(&T, &P)> {
        match &self.current_friend_group {
            None => self.underlying.peek().map(|(gp, p)| {
                let z = &self
                    .shibboleth_to_friends
                    .get(gp)
                    .expect("all shibboleths in priority queue have corresponding items")
                    .head;
                (z, p)
            }),
            Some((real_group, top_priority)) => Some((&real_group.head, top_priority)),
        }
    }

    fn my_enqueue(&mut self, new_obj: T, new_obj_priority: P) {
        let my_shibboleth = new_obj.friendship_shibboleth();
        if let Some(old_priority) = self.underlying.get_priority(&my_shibboleth) {
            // found friends waiting in line
            let (priority_changed, new_priority) =
                (self.priority_combiner)(old_priority, &new_obj_priority);
            if priority_changed {
                // the priority can make this group the head of the waiting
                // but it can't push them up to be past the group that is currently entering
                // they are already in the ``foyer``
                let _overwritten = self
                    .underlying
                    .change_priority(&my_shibboleth, new_priority);
            }
            if let Some(friend_grp) = self.shibboleth_to_friends.get_mut(&my_shibboleth) {
                friend_grp.push(new_obj);
            } else {
                panic!("found shibboleth in priority queue but not the corresponding friends");
            }
            self.waiting_len += 1;
        } else if let Some(head_group) = &mut self.current_friend_group {
            // there is a head group entering right now, they might be friends
            if head_group.0.head.friendship_shibboleth() == my_shibboleth {
                // they are your friends and they are the ones just going in now
                head_group.0.push(new_obj);
                let (priority_changed, new_priority) =
                    (self.priority_combiner)(&head_group.1, &new_obj_priority);
                if priority_changed {
                    head_group.1 = new_priority;
                }
            } else {
                // they are not your friends, go into the line
                // even if you have higher priority than them
                self.underlying
                    .push(my_shibboleth.clone(), new_obj_priority);
                self.shibboleth_to_friends
                    .insert(my_shibboleth, nonempty::nonempty![new_obj]);
                self.waiting_len += 1;
            }
        } else {
            // there is no group currently being let in, and you have no friends in line
            self.underlying
                .push(my_shibboleth.clone(), new_obj_priority);
            self.shibboleth_to_friends
                .insert(my_shibboleth, nonempty::nonempty![new_obj]);
            self.waiting_len += 1;
        }
    }

    fn enqueue_batch(&mut self, new_batch: impl IntoIterator<Item = T>, new_batch_priority: P) {
        if let Some(new_batch) = NonEmpty::from_vec(new_batch.into_iter().collect()) {
            let (all_friends, the_shibboleth) = are_all_friends(&new_batch);
            if !all_friends {
                for z in new_batch {
                    self.my_enqueue(z, new_batch_priority.clone());
                }
                return;
            }
            let new_len = new_batch.len();
            if let Some(old_priority) = self.underlying.get_priority(&the_shibboleth) {
                // found more friends waiting in line
                let (priority_changed, new_priority) =
                    (self.priority_combiner)(old_priority, &new_batch_priority);
                if priority_changed {
                    // the priority can make this group the head of the waiting
                    // but it can't push them up to be past the group that is currently entering
                    // they are already in the ``foyer``
                    let _overwritten = self
                        .underlying
                        .change_priority(&the_shibboleth, new_priority);
                }
                if let Some(friend_grp) = self.shibboleth_to_friends.get_mut(&the_shibboleth) {
                    friend_grp.extend(new_batch);
                } else {
                    panic!("found shibboleth in priority queue but not the corresponding friends");
                }
                self.waiting_len += new_len;
            } else if let Some(head_group) = &mut self.current_friend_group {
                // there is a head group entering right now, they might be friends
                if head_group.0.head.friendship_shibboleth() == the_shibboleth {
                    // they are your friends and they are the ones just going in now
                    head_group.0.extend(new_batch);
                    let (priority_changed, new_priority) =
                        (self.priority_combiner)(&head_group.1, &new_batch_priority);
                    if priority_changed {
                        head_group.1 = new_priority;
                    }
                } else {
                    // they are not your friends, go into the line
                    // even if you have higher priority than them
                    self.underlying
                        .push(the_shibboleth.clone(), new_batch_priority);
                    self.shibboleth_to_friends.insert(the_shibboleth, new_batch);
                    self.waiting_len += new_len;
                }
            } else {
                // there is no group currently being let in, and you have no friends in line
                self.underlying
                    .push(the_shibboleth.clone(), new_batch_priority);
                self.shibboleth_to_friends.insert(the_shibboleth, new_batch);
                self.waiting_len += new_len;
            }
        }
    }

    fn my_dequeue(&mut self) -> Option<(T, P)> {
        let taken_current_friend_group = self.current_friend_group.take();
        if let Some((mut head_group, head_priority)) = taken_current_friend_group {
            if head_group.len() == 1 {
                Some((head_group.head, head_priority))
            } else {
                let ret_val = head_group.pop().map(|z| (z, head_priority.clone()));
                self.current_friend_group = Some((head_group, head_priority));
                ret_val
            }
        } else {
            let new_head_group_shibboleth = self.underlying.pop();
            if let Some((shibboleth, priority)) = new_head_group_shibboleth {
                // the next batch becomes the currently processing group
                let head_of_line = self.shibboleth_to_friends.remove(&shibboleth);
                if let Some(new_head_of_line) = head_of_line {
                    self.waiting_len -= new_head_of_line.len();
                    self.current_friend_group = Some((new_head_of_line, priority));
                    self.my_dequeue()
                } else {
                    panic!("found shibboleth in priority queue but not the corresponding friends");
                }
            } else {
                // the entire line is empty
                None
            }
        }
    }

    fn my_len(&self) -> usize {
        let at_head_len = self.current_friend_group.as_ref().map_or(0, |z| z.0.len());
        at_head_len + self.waiting_len
    }

    fn is_empty(&self) -> bool {
        self.waiting_len == 0
            && self
                .current_friend_group
                .as_ref()
                .is_none_or(|z| z.0.is_empty())
    }

    fn dequeue_batch(&mut self, around_how_many: usize, hard_limit: usize) -> Vec<(T, P)> {
        let mut to_return = Vec::with_capacity(around_how_many);
        for _ in 0..around_how_many {
            let put_in = self.israeli_dequeue_batch();
            if let Some(real_put_in) = put_in {
                to_return.extend(
                    real_put_in
                        .0
                        .into_iter()
                        .map(|z| (z, real_put_in.1.clone())),
                );
            } else {
                break;
            }
            if to_return.len() >= around_how_many {
                break;
            }
        }
        if to_return.len() > hard_limit {
            let left_back = to_return.split_off(hard_limit);
            if let Some(nonempty_left_back) = NonEmpty::from_vec(left_back) {
                let all_same_priority = nonempty_left_back
                    .iter()
                    .all(|(_, z)| *z == nonempty_left_back.head.1);
                if all_same_priority {
                    let (new_batch_head, new_batch_priority) = nonempty_left_back.head;
                    let the_shibboleth = new_batch_head.friendship_shibboleth();
                    let mut new_batch = NonEmpty::new(new_batch_head);
                    new_batch.extend(nonempty_left_back.tail.into_iter().map(|z| z.0));
                    let all_friends = new_batch
                        .tail
                        .iter()
                        .all(|r| r.friendship_shibboleth() == the_shibboleth);
                    if self.current_friend_group.is_none() || all_friends {
                        // the last friend group was too big, some of them go back to the head of the line
                        self.current_friend_group = Some((new_batch, new_batch_priority));
                    } else {
                        // there is a group in the ``foyer``, enqueue them as normal
                        // or the extras were not all part of 1 friend group
                        // this should not occur
                        self.enqueue_batch(new_batch, new_batch_priority);
                    }
                } else {
                    // the extras were not all a part of one group for some reason
                    // this should not occur
                    for (a, b) in nonempty_left_back {
                        self.my_enqueue(a, b);
                    }
                }
            }
        }
        to_return
    }
}

mod test {
    // TODO nontrivial tests

    use super::Friendly;

    const MY_U8_FREINDLINESS: u8 = 5;

    #[allow(dead_code)]
    #[derive(PartialEq, Eq, Debug)]
    #[repr(transparent)]
    struct MyU8(u8);
    impl Friendly<u8> for MyU8 {
        fn friendship_shibboleth(&self) -> u8 {
            self.0 % MY_U8_FREINDLINESS
        }
    }

    #[test]
    fn nothing_in_out() {
        use super::IsraeliPriority;
        use crate::my_priority_queue::AbstractPriorityQueue;

        let mut q = IsraeliPriority::<MyU8, u8, u8>::with_capacity(2);
        assert_eq!(q.my_len(), 0);
        assert!(q.is_empty());
        assert!(q.my_dequeue().is_none());
        assert!(q.dequeue_batch(10, 10).is_empty());
    }

    #[test]
    fn same_order_inout() {
        use super::IsraeliPriority;
        use crate::my_priority_queue::AbstractPriorityQueue;
        for max_num in 0..=MY_U8_FREINDLINESS {
            let mut q = IsraeliPriority::<MyU8, u8, u8>::with_capacity(max_num as usize);
            for (a, b) in std::iter::zip(0..max_num, (0..max_num).rev()) {
                q.my_enqueue(MyU8(a), b);
            }
            assert_eq!(q.my_len(), max_num.into());
            if max_num > 0 {
                assert!(!q.is_empty());
                let removed = q.my_dequeue();
                assert_eq!(removed, Some((MyU8(0), max_num - 1)));
            } else {
                assert!(q.is_empty());
                let removed = q.my_dequeue();
                assert_eq!(removed, None);
            }
            assert_eq!(
                q.dequeue_batch(10, 10),
                (1..max_num)
                    .map(|z| (MyU8(z), max_num - 1 - z))
                    .collect::<Vec<_>>()
            );

            let mut q = IsraeliPriority::<MyU8, u8, u8>::with_capacity(max_num as usize);
            for (a, b) in std::iter::zip(0..max_num, (0..max_num).rev()) {
                q.my_enqueue(MyU8(a), b);
            }
            assert_eq!(q.my_len(), max_num.into());
            assert!(!q.is_empty() || max_num == 0);
            for idx in 0..max_num {
                assert!(q.current_friend_group.is_none());
                let removed = q.my_dequeue();
                assert_eq!(removed, Some((MyU8(idx), max_num - 1 - idx)));
                assert!(q.current_friend_group.is_none());
            }
            assert!(q.is_empty());
            let removed = q.my_dequeue();
            assert_eq!(removed, None);
            assert!(q.is_empty());
        }
    }
}
