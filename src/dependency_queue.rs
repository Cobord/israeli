// TODO test

use petgraph::{graph::NodeIndex, Graph};
use std::cmp::Ordering;

use crate::my_priority_queue::AbstractPriorityQueue;

pub trait Blocker {
    /// if self was enqueued before other
    /// do they have to be dequeued in that order too
    fn am_i_blocker(&self, other: &Self) -> bool;
}

/// imagine there are commands that are
/// enqueued in arbitrary order and we are using priorities
/// to shuffle around which order they get dequeue'd
/// but there are some that still have to occur before others
/// regardless of priorities
pub struct BlockingQueue<T, P>
where
    T: Blocker,
    P: Ord,
{
    my_dag: Graph<(T, P), ()>,
    srcs: Vec<NodeIndex>,
    sinks: Vec<NodeIndex>,
}

impl<T, P> AbstractPriorityQueue<T, P> for BlockingQueue<T, P>
where
    T: Blocker + Clone,
    P: Ord + Clone,
{
    fn empty_copy(&self) -> Self {
        let num_nodes = self.my_len();
        let num_edges = self.my_dag.edge_count();
        let num_srcs = self.srcs.len();
        let num_snks = self.sinks.len();
        Self {
            my_dag: Graph::<_, _>::with_capacity(num_nodes, num_edges),
            srcs: Vec::with_capacity(num_srcs),
            sinks: Vec::with_capacity(num_snks),
        }
    }

    fn my_peek(&self) -> Option<(&T, &P)> {
        let mut srcs = self.srcs.iter().map(|id| {
            self.my_dag
                .node_weight(*id)
                .expect("srcs are all valid nodes")
        });
        if let Some((first_src_item, first_src_priority)) = srcs.next() {
            let to_return = srcs.fold((first_src_item, first_src_priority), |acc, next| {
                if acc.1.cmp(&next.1) == Ordering::Less {
                    (&next.0, &next.1)
                } else {
                    acc
                }
            });
            Some(to_return)
        } else {
            None
        }
    }

    fn my_enqueue(&mut self, new_obj: T, new_obj_priority: P) {
        let cur_node_idx = self
            .my_dag
            .add_node((new_obj.clone(), new_obj_priority.clone()));
        let mut new_sinks = std::mem::take(&mut self.sinks);
        let mut removals = Vec::new();
        for (cur_sink_idx, cur_sink) in new_sinks.iter().enumerate() {
            let cur_item = self
                .my_dag
                .node_weight(*cur_sink)
                .expect("sinks exist in graph");
            if cur_item.0.am_i_blocker(&new_obj) {
                removals.push(cur_sink_idx);
                self.my_dag.add_edge(*cur_sink, cur_node_idx, ());
            }
        }
        if removals.is_empty() {
            let mut is_cur_also_src = true;
            for temp_node_idx in self.my_dag.node_indices() {
                let temp_item = &self
                    .my_dag
                    .node_weight(temp_node_idx)
                    .expect("Already checked exists")
                    .0;
                if temp_item.am_i_blocker(&new_obj) {
                    is_cur_also_src = false;
                    self.my_dag.add_edge(temp_node_idx, cur_node_idx, ());
                }
            }
            if is_cur_also_src {
                self.srcs.push(cur_node_idx);
            }
        } else {
            removals.sort();
            for removal in removals.iter().rev() {
                new_sinks.remove(*removal);
            }
        }
        new_sinks.push(cur_node_idx);
        self.sinks = new_sinks;
    }

    fn enqueue_batch(&mut self, new_batch: impl IntoIterator<Item = T>, new_batch_priority: P) {
        for item in new_batch {
            self.my_enqueue(item, new_batch_priority.clone());
        }
    }

    fn my_dequeue(&mut self) -> Option<(T, P)> {
        let new_srcs_1 = std::mem::take(&mut self.srcs);
        let mut new_srcs_2 = new_srcs_1.clone();
        let srcs = new_srcs_1
            .iter()
            .map(|s| (s, self.my_dag.node_weight(*s).expect("srcs exist")));
        let mut highest_priority_info = None;
        for (which_src, (cur_src_idx, (cur_src, cur_src_priority))) in srcs.enumerate() {
            if let Some((_, _, _, highest_priority)) = &highest_priority_info {
                if cur_src_priority.cmp(highest_priority) == Ordering::Greater {
                    highest_priority_info =
                        Some((which_src, cur_src_idx, cur_src, cur_src_priority.clone()));
                }
            } else {
                highest_priority_info =
                    Some((which_src, cur_src_idx, cur_src, cur_src_priority.clone()));
            }
        }
        if let Some((z0, z1, _z2, _z3)) = highest_priority_info {
            new_srcs_2.remove(z0);
            let neighbor_idces = self
                .my_dag
                .neighbors_directed(*z1, petgraph::Direction::Outgoing);
            for neighbor in neighbor_idces {
                let count_incoming = self
                    .my_dag
                    .neighbors_directed(neighbor, petgraph::Direction::Incoming)
                    .count();
                if count_incoming <= 1 {
                    new_srcs_2.push(neighbor);
                }
            }
            let pulled_out = self.my_dag.remove_node(*z1);
            self.srcs = new_srcs_2;
            pulled_out
        } else {
            None
        }
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

    fn my_len(&self) -> usize {
        self.my_dag.node_count()
    }

    fn is_empty(&self) -> bool {
        self.my_len() == 0
    }
}

mod test {

    #[test]
    fn presence() {
        // TODO stub
    }
}
