use std::cmp::Ordering;
use std::fmt::Debug;

extern crate arrayvec;
use arrayvec::ArrayVec;

#[derive(Debug)]
pub struct Node<T, const M: usize>
where
    [(); M - 1]: Sized,
{
    idx: usize,
    parent: Option<usize>,
    values: ArrayVec<T, { M - 1 }>,
    children: ArrayVec<usize, M>,
}

impl<T, const M: usize> Default for Node<T, M>
where
    [(); M - 1]: Sized,
{
    fn default() -> Self {
        Node {
            idx: 0,
            parent: None,
            values: ArrayVec::new(),
            children: ArrayVec::new(),
        }
    }
}

impl<T, const M: usize> Node<T, M>
where
    [(); M - 1]: Sized,
{
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug)]
pub struct Tree<T, const M: usize>
where
    [(); M - 1]: Sized,
{
    arena: Vec<Node<T, M>>,
    root_id: usize,
}

impl<T, const M: usize> Default for Tree<T, M>
where
    T: Ord + Copy + Default + Debug,
    [(); M - 1]: Sized,
{
    fn default() -> Self {
        let mut t = Tree::<T, M> {
            root_id: 0,
            arena: Vec::new(),
        };
        let root_id = t.arena.len();
        let root = Node::<T, M> {
            idx: root_id,
            ..Default::default()
        };
        t.arena.push(root);
        t.root_id = root_id;
        t
    }
}

impl<T, const M: usize> Tree<T, M>
where
    T: Ord + Copy + Default + Debug,
    [(); M - 1]: Sized,
{
    fn insert_into(&mut self, cur_id: usize, value: T) -> Option<(T, usize)> {
        let cur = &self.arena[cur_id];
        let insert_idx = {
            let mut low = 0;
            let mut high = cur.values.len();
            let mut median = ((high - low) / 2) + low;
            while low < high {
                match value.cmp(&cur.values[median]) {
                    Ordering::Less => high = median,
                    Ordering::Equal => return None,
                    Ordering::Greater => low = median + 1,
                };
                median = ((high - low) / 2) + low;
            }
            median
        };
        debug!(&cur, value, insert_idx);

        let (value, value_right_child_id) = if cur.is_leaf() {
            (value, None)
        } else {
            let child_id = self.arena[cur_id].children[insert_idx];
            if let Some((median, median_right_child_id)) = self.insert_into(child_id, value) {
                (median, Some(median_right_child_id))
            } else {
                return None;
            }
        };

        #[cfg(debug_assertions)]
        if let Some(child_id) = value_right_child_id {
            debug!(value, &self.arena[child_id], &self.arena[cur_id]);

            assert_eq!(
                self.arena[cur_id].values.len() + 1,
                self.arena[cur_id].children.len()
            );

            if !self.arena[child_id].is_leaf() {
                assert_eq!(
                    self.arena[child_id].values.len() + 1,
                    self.arena[child_id].children.len()
                );
            }
        }

        let cur = &self.arena[cur_id];
        if !cur.values.is_full() {
            let cur = &mut self.arena[cur_id];
            cur.values.insert(insert_idx, value);
            if let Some(child_id) = value_right_child_id {
                cur.children.insert(insert_idx + 1, child_id);
                self.arena[child_id].parent = Some(cur_id);
            }
            None
        } else {
            // need to separate node
            let (right, median) = {
                let right_id = self.arena.len();
                let mut right = Node::<T, M> {
                    idx: right_id,
                    ..Default::default()
                };
                let cur = &mut self.arena[cur_id];
                match insert_idx.cmp(&(M / 2)) {
                    Ordering::Greater => {
                        // values: | left | median | right |

                        // left: 0..M / 2
                        // M / 2
                        // median: M / 2
                        // right: M / 2 + 1..M - 1

                        // right: | part one | value | part two |
                        // part one: M / 2 + 1..insert_idx
                        // value
                        // part two: insert_idx..M - 1
                        // M / 2 - 1
                        {
                            let drain = &mut cur.values.drain(M / 2 + 1..);
                            right.values.extend(drain.take(insert_idx - M / 2 - 1));
                            right.values.push(value);
                            right.values.extend(drain);
                        };
                        // children: | left | right |
                        // left: 0..M / 2 + 1
                        // M / 2 + 1
                        // right: M / 2 + 1..
                        // M / 2

                        if let Some(child_id) = value_right_child_id {
                            {
                                let drain = &mut cur.children.drain(M / 2 + 1..);
                                right.children.extend(drain.take(insert_idx - M / 2));
                                right.children.push(child_id);
                                right.children.extend(drain);
                            }
                            for &child_id in &right.children {
                                self.arena[child_id].parent = Some(right_id);
                            }
                        }

                        (right, self.arena[cur_id].values.remove(M / 2))
                    }
                    Ordering::Less => {
                        // values: | left | median | right |

                        // left: | part one | value | part two |
                        // part one: 0..insert_idx
                        // value
                        // part two: insert_idx..M / 2 - 1
                        // M / 2

                        // median: M / 2 - 1
                        // right: M / 2..
                        // M / 2 - 1

                        right.values.extend(cur.values.drain(M / 2..));
                        let median = cur.values.remove(M / 2 - 1);
                        cur.values.insert(insert_idx, value);

                        // children: | left | right |

                        // left: | part one | child_id | part two |
                        // part one: 0..insert_idx + 1
                        // child_id: insert_idx + 1
                        // part two: insert_idx + 1..M / 2
                        // M / 2 + 1

                        // right: M / 2 ..
                        // M / 2

                        if let Some(child_id) = value_right_child_id {
                            right.children.extend(cur.children.drain(M / 2..));
                            for &child_id in &right.children {
                                self.arena[child_id].parent = Some(right_id);
                            }

                            let cur = &mut self.arena[cur_id];
                            cur.children.insert(insert_idx + 1, child_id);
                            self.arena[child_id].parent = Some(cur_id);
                        }

                        (right, median)
                    }
                    Ordering::Equal => {
                        // values: | left | median | right |
                        // left: 0..M / 2
                        // M / 2
                        // median: value
                        // right: M / 2..M - 1
                        // M / 2 - 1

                        right.values.extend(cur.values.drain(M / 2..));

                        // children: | left | right |

                        // left: 0..M / 2
                        // child_id
                        // M / 2 + 1

                        // right: M / 2..
                        // M / 2

                        if let Some(child_id) = value_right_child_id {
                            right.children.push(child_id);
                            right.children.extend(cur.children.drain(M / 2 + 1..));
                            for &child_id in &right.children {
                                self.arena[child_id].parent = Some(right_id);
                            }
                        }

                        (right, value)
                    }
                }
            };
            assert_eq!(right.idx, self.arena.len());
            self.arena.push(right);
            Some((median, self.arena.len() - 1))
        }
    }

    pub fn insert(&mut self, value: T) {
        debug!(value);
        if let Some((median, right_id)) = self.insert_into(self.root_id, value) {
            debug!(median, &self.arena[right_id], &self.arena[self.root_id]);
            let root_id = self.arena.len();
            let mut root = Node::<T, M> {
                idx: root_id,
                ..Default::default()
            };
            root.values.push(median);
            root.children.push(self.root_id);
            root.children.push(right_id);

            self.arena[self.root_id].parent = Some(root_id);
            self.arena[right_id].parent = Some(root_id);

            self.arena.push(root);
            self.root_id = root_id;
        };
    }

    pub fn delete(&mut self, val: T) -> bool {
        match self.search(val) {
            (_, _, false) => false,
            (node_id, value_idx, true) => {
                let node_id = self.delete_value(node_id, value_idx);
                self.rebalance(node_id);
                true
            }
        }
    }

    fn delete_value(&mut self, node_id: usize, value_idx: usize) -> usize {
        {
            let node = &mut self.arena[node_id];
            node.values.remove(value_idx);
        }
        let node = &self.arena[node_id];
        if node.is_leaf() {
            node_id
        } else {
            // The internal node needs to find a value from children to fill the void.
            let (from_id, from_value_idx) = match self.adjacent_children(node_id, value_idx) {
                (Some(&left_id), None) => {
                    let (most_right_id, _) = self.most_right(left_id);
                    (most_right_id, self.arena[most_right_id].values.len() - 1)
                }
                (Some(&left_id), Some(&right_id)) => {
                    let (most_right_id, most_right_depth) = self.most_right(left_id);
                    let (most_left_id, most_left_depth) = self.most_left(right_id);
                    // Taking the value from the less depth node
                    // can rebalance faster than taking the other way.
                    if most_left_depth > most_right_depth {
                        (most_right_id, self.arena[most_right_id].values.len() - 1)
                    } else {
                        (most_left_id, 0)
                    }
                }
                (None, Some(&right_id)) => {
                    let (most_left_id, _) = self.most_left(right_id);
                    (most_left_id, 0)
                }
                (None, None) => unreachable!(),
            };

            let new_separator_value = {
                let from_node = &mut self.arena[from_id];
                from_node.values.remove(from_value_idx)
            };

            let node = &mut self.arena[node_id];
            node.values.insert(value_idx, new_separator_value);

            // Return the leaf node for rebalancing later
            from_id
        }
    }

    fn rebalance(&mut self, node_id: usize) {
        let mut cur_id = node_id;
        loop {
            let node = &self.arena[cur_id];
            if node.is_root() || node.values.len() >= (M - 1) / 2 {
                // Check the node is deficient or not, except the root node.
                return;
            }
            let (left, _cur_idx, right) = self.sibling(cur_id);
            let right_len = if let Some(id) = right {
                self.arena[id].values.len()
            } else {
                0
            };
            let left_len = if let Some(id) = left {
                self.arena[id].values.len()
            } else {
                0
            };

            let (max_len, is_left_max) = if left_len > right_len {
                (left_len, true)
            } else {
                (right_len, false)
            };

            if max_len > (M - 1) / 2 {
                if is_left_max {
                    self.rotate_right(cur_id);
                } else {
                    self.rotate_left(cur_id);
                }
                return;
            }

            let (merged_node_id, _deficient_node_id) = if right_len > 0 && left_len > 0 {
                // Node merges a minor sibling node
                if is_left_max {
                    self.merge_right(cur_id)
                } else {
                    self.merge_left(cur_id)
                }
            } else if right_len > 0 {
                // left_len == 0
                self.merge_right(cur_id)
            } else if left_len > 0 {
                // right_len == 0
                self.merge_left(cur_id)
            } else {
                unreachable!()
            };

            let parent_id = self.arena[cur_id].parent.unwrap();
            let parent = &self.arena[parent_id];
            if parent.values.is_empty() && parent.is_root() {
                self.arena[merged_node_id].parent = None;
                self.root_id = merged_node_id;
                return;
            }
            cur_id = parent_id;
        }
    }

    fn rotate_left(&mut self, node_id: usize) {
        let parent_id = self.arena[node_id].parent.unwrap();
        if let (_, Some(node_idx), Some(right_id)) = self.sibling(node_id) {
            let value_idx = node_idx;
            let separator = self.arena[parent_id].values.remove(value_idx);
            self.arena[node_id].values.push(separator);
            let new_separator = self.arena[right_id].values.remove(0);
            self.arena[parent_id]
                .values
                .insert(value_idx, new_separator);

            // rotate with children if exists.
            let right = &mut self.arena[right_id];
            if !right.children.is_empty() {
                let child_id = right.children.remove(0);
                self.arena[node_id].children.push(child_id);
                self.arena[child_id].parent = Some(node_id);
            }
        } else {
            unreachable!()
        };
    }

    fn rotate_right(&mut self, node_id: usize) {
        let parent_id = self.arena[node_id].parent.unwrap();
        if let (Some(left_id), Some(node_idx), _) = self.sibling(node_id) {
            let value_idx = node_idx - 1;
            let separator = self.arena[parent_id].values.remove(value_idx);
            self.arena[node_id].values.push(separator);
            let new_separator = {
                let left = &mut self.arena[left_id];
                left.values.remove(left.values.len() - 1)
            };
            self.arena[parent_id]
                .values
                .insert(value_idx, new_separator);

            // rotate with children if exists.
            let left = &mut self.arena[left_id];
            if !left.children.is_empty() {
                let child_id = left.children.remove(left.children.len() - 1);
                self.arena[node_id].children.insert(0, child_id);
                self.arena[child_id].parent = Some(node_id);
            }
        } else {
            unreachable!()
        };
    }

    fn merge_left(&mut self, node_id: usize) -> (usize, usize) {
        if let (Some(left_id), Some(node_idx), _) = self.sibling(node_id) {
            self.merge_sibling_nodes(left_id, node_idx - 1, node_id);
            (left_id, node_id)
        } else {
            unreachable!()
        }
    }

    fn merge_right(&mut self, node_id: usize) -> (usize, usize) {
        if let (_, Some(node_idx), Some(right_id)) = self.sibling(node_id) {
            self.merge_sibling_nodes(node_id, node_idx, right_id);
            (node_id, right_id)
        } else {
            unreachable!()
        }
    }

    fn merge_sibling_nodes(&mut self, node_id: usize, separator_idx: usize, right_id: usize) {
        let parent_id = self.arena[node_id].parent.unwrap();

        let parent = &mut self.arena[parent_id];
        let separator = parent.values.remove(separator_idx);
        self.arena[node_id].values.push(separator);
        self.arena[parent_id].children.remove(separator_idx + 1);

        let right_values = self.arena[right_id].values.take();
        self.arena[node_id].values.extend(right_values);
        if !self.arena[right_id].children.is_empty() {
            let right_children = self.arena[right_id].children.take();
            for &child_id in right_children.iter() {
                self.arena[child_id].parent = Some(node_id);
            }
            self.arena[node_id].children.extend(right_children);
        }
    }

    fn sibling(&self, node_id: usize) -> (Option<usize>, Option<usize>, Option<usize>) {
        let node = &self.arena[node_id];
        match node.parent {
            None => (None, None, None),
            Some(parent_id) => {
                let parent = &self.arena[parent_id];
                let mut node_child_idx = parent.children.len();
                for (idx, &child_id) in parent.children.iter().enumerate() {
                    if child_id == node_id {
                        node_child_idx = idx;
                        break;
                    }
                }
                (
                    if node_child_idx == 0 {
                        None
                    } else {
                        Some(parent.children[node_child_idx - 1])
                    },
                    Some(node_child_idx),
                    if node_child_idx + 1 < parent.children.len() {
                        Some(parent.children[node_child_idx + 1])
                    } else {
                        None
                    },
                )
            }
        }
    }

    fn adjacent_children(
        &self,
        node_id: usize,
        value_idx: usize,
    ) -> (Option<&usize>, Option<&usize>) {
        let cur = &self.arena[node_id];
        if value_idx < cur.children.len() {
            (
                Some(&cur.children[value_idx]),
                if value_idx < cur.children.len() - 1 {
                    Some(&cur.children[value_idx + 1])
                } else {
                    None
                },
            )
        } else {
            unreachable!();
        }
    }

    fn most_left(&self, node_id: usize) -> (usize, usize) {
        let mut depth = 0;
        let mut cur = &self.arena[node_id];
        while let Some(&id) = cur.children.first() {
            cur = &self.arena[id];
            depth += 1;
        }
        (cur.idx, depth)
    }

    fn most_right(&self, node_id: usize) -> (usize, usize) {
        let mut depth = 0;
        let mut cur = &self.arena[node_id];
        while let Some(&id) = cur.children.last() {
            cur = &self.arena[id];
            depth += 1;
        }
        (cur.idx, depth)
    }

    /// locate the (parent_value_idx, node_id, value_idx) for inserting the value.
    fn search(&self, val: T) -> (usize, usize, bool) {
        let mut cur = &self.arena[self.root_id];
        loop {
            // find the insert index of value in the current node values
            let insert_idx = {
                let mut low = 0;
                let mut high = cur.values.len();
                let mut median = ((high - low) / 2) + low;
                while low < high {
                    match val.cmp(&cur.values[median]) {
                        Ordering::Less => high = median,
                        Ordering::Equal => return (cur.idx, median, true),
                        Ordering::Greater => low = median + 1,
                    };
                    median = ((high - low) / 2) + low;
                }
                median
            };

            // try to find a space for inserting the value from the left sub-tree
            if !cur.is_leaf() {
                cur = &self.arena[cur.children[insert_idx]];
                continue;
            }
            return (cur.idx, insert_idx, false);
        }
    }

    pub fn range(&self, _begin: T, _end: T) -> Vec<T> {
        todo!();
    }

    pub fn get(&self, value: T) -> Option<T> {
        let mut cur = &self.arena[self.root_id];
        loop {
            let insert_idx = {
                let mut low = 0;
                let mut high = cur.values.len();
                let mut median = ((high - low) / 2) + low;
                while low < high {
                    match value.cmp(&cur.values[median]) {
                        Ordering::Less => high = median,
                        Ordering::Equal => return Some(cur.values[median]),
                        Ordering::Greater => low = median + 1,
                    };
                    median = ((high - low) / 2) + low;
                }
                median
            };
            if !cur.is_leaf() {
                cur = &self.arena[cur.children[insert_idx]];
                continue;
            }
            return None;
        }
    }

    pub fn traversal_bfs(&self) -> Vec<T> {
        use std::collections::VecDeque;
        let mut q = VecDeque::with_capacity(self.arena.len());
        let mut cur = &self.arena[self.root_id];

        let mut path = Vec::new();
        loop {
            for &val in cur.values.iter() {
                path.push(val);
            }
            for &child_id in cur.children.iter() {
                q.push_back(child_id);
            }
            match q.pop_front() {
                Some(id) => cur = &self.arena[id],
                None => break,
            }
        }
        path
    }

    pub fn format_debug(&self) -> String {
        if self.arena.is_empty() {
            return String::from("[]");
        }
        use std::collections::VecDeque;
        let mut q = VecDeque::with_capacity(self.arena.len());
        let mut cur = &self.arena[self.root_id];

        let mut path = Vec::new();
        let mut depth = 0;
        loop {
            if depth >= path.len() {
                path.push(Vec::new());
            }
            path[depth].push((cur.idx, &cur.values));
            for &child_id in cur.children.iter() {
                q.push_back((child_id, depth + 1));
            }
            match q.pop_front() {
                Some((id, _depth)) => {
                    cur = &self.arena[id];
                    depth = _depth
                }
                None => break,
            }
        }
        let mut rv = String::new();
        for row in path.iter() {
            for (idx, n) in row.iter() {
                rv.push_str(&format!("#{:?}{:?} ", idx, n));
            }
            rv.pop();
            rv.push('\n');
        }
        rv.pop();
        rv
    }
}

#[test]
fn insert_root() {
    let mut t = Tree::<_, 3>::default();
    t.insert(1);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.values.as_slice(), vec![1]);
}

#[test]
fn insert_1() {
    let mut t = Tree::<_, 3>::default();
    t.insert(9);
    t.insert(10);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.values.as_slice(), vec![9, 10]);

    let mut t = Tree::<_, 3>::default();
    t.insert(10);
    t.insert(9);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.values.as_slice(), vec![9, 10]);
}

#[test]
fn insert_2() {
    let mut t = Tree::<_, 4>::default();
    t.insert(9);
    t.insert(11);
    t.insert(10);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.values.as_slice(), vec![9, 10, 11]);
}

#[test]
fn insert_3() {
    let mut t = Tree::<_, 3>::default();
    t.insert(9);
    debug!(t.format_debug());
    t.insert(10);
    debug!(t.format_debug());
    t.insert(0);
    debug!(t.format_debug());
    assert_eq!(t.arena.len(), 3);
    let root = &t.arena[t.root_id];
    debug!(t.format_debug());
    assert_eq!(root.values.as_slice(), vec![9]);
    assert_eq!(root.children.as_slice(), vec![0, 1]);
    assert_eq!(t.arena[0].values.as_slice(), vec![0]);
    assert_eq!(t.arena[1].values.as_slice(), vec![10]);
}

#[test]
fn insert_4() {
    let mut t = Tree::<_, 3>::default();
    for &val in vec![1, 2, 3].iter() {
        t.insert(val);
    }
    assert_eq!(t.format_debug(), "#2[2]\n#0[1] #1[3]");

    t.insert(4);
    assert_eq!(t.format_debug(), "#2[2]\n#0[1] #1[3, 4]");

    t.insert(5);
    assert_eq!(t.format_debug(), "#2[2, 4]\n#0[1] #1[3] #3[5]");

    t.insert(6);
    assert_eq!(t.format_debug(), "#2[2, 4]\n#0[1] #1[3] #3[5, 6]");

    t.insert(7);
    assert_eq!(
        t.format_debug(),
        "#6[4]\n#2[2] #5[6]\n#0[1] #1[3] #3[5] #4[7]"
    );
}

#[test]
fn insert_5() {
    let mut t = Tree::<_, 4>::default();
    for &val in vec![1, 2, 3].iter() {
        t.insert(val);
    }
    assert_eq!(t.format_debug(), "#0[1, 2, 3]");

    t.insert(4);
    assert_eq!(t.format_debug(), "#2[3]\n#0[1, 2] #1[4]");

    for &val in vec![5, 6].iter() {
        t.insert(val);
    }
    assert_eq!(t.format_debug(), "#2[3]\n#0[1, 2] #1[4, 5, 6]");

    t.insert(7);
    assert_eq!(t.format_debug(), "#2[3, 6]\n#0[1, 2] #1[4, 5] #3[7]");

    for &val in vec![8, 9].iter() {
        t.insert(val);
    }
    assert_eq!(t.format_debug(), "#2[3, 6]\n#0[1, 2] #1[4, 5] #3[7, 8, 9]");

    t.insert(10);
    assert_eq!(
        t.format_debug(),
        "#2[3, 6, 9]\n#0[1, 2] #1[4, 5] #3[7, 8] #4[10]"
    );

    for &val in vec![11, 12].iter() {
        t.insert(val);
    }
    assert_eq!(
        t.format_debug(),
        "#2[3, 6, 9]\n#0[1, 2] #1[4, 5] #3[7, 8] #4[10, 11, 12]"
    );
    t.insert(13);
    assert_eq!(
        t.format_debug(),
        "#7[9]\n#2[3, 6] #6[12]\n#0[1, 2] #1[4, 5] #3[7, 8] #4[10, 11] #5[13]"
    );
}

#[test]
fn traversal_bfs() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..8 {
        t.insert(val);
    }

    assert_eq!(t.traversal_bfs(), vec![4, 2, 6, 1, 3, 5, 7]);
}

#[test]
fn format_debug_1() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#6[4]
#2[2] #5[6]
#0[1] #1[3] #3[5] #4[7]"
    );
}

#[test]
fn format_debug_2() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..7 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[5, 6]"
    );
}

#[test]
fn format_debug_3() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..6 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[5]"
    );
}

#[test]
fn format_debug_4() {
    let t = Tree::<usize, 3>::default();
    assert_eq!(t.format_debug(), "#0[]");
}

#[test]
fn delete_notfound() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..4 {
        t.insert(val);
    }
    assert_eq!(t.delete(4), false);
}

#[test]
fn delete_1() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..5 {
        t.insert(val);
    }
    for val in 6..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[6, 7]"
    );

    t.delete(6);

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[7]"
    );
}

#[test]
fn delete_2() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..5 {
        t.insert(val);
    }
    for val in 6..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[6, 7]"
    );

    t.delete(4);

    assert_eq!(
        t.format_debug(),
        "#2[2, 6]
#0[1] #1[3] #3[7]"
    );
}

#[test]
fn delete_3() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..5 {
        t.insert(val);
    }
    for val in 6..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[6, 7]"
    );

    t.delete(3);

    assert_eq!(
        t.format_debug(),
        "#2[2, 6]
#0[1] #1[4] #3[7]"
    );
}

#[test]
fn delete_4() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..5 {
        t.insert(val);
    }
    for val in 6..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[6, 7]"
    );

    t.delete(1);

    assert_eq!(
        t.format_debug(),
        "#2[4]
#0[2, 3] #3[6, 7]"
    );
}

#[test]
fn delete_5() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..5 {
        t.insert(val);
    }
    t.insert(6);

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[6]"
    );

    t.delete(3);

    assert_eq!(
        t.format_debug(),
        "#2[4]
#0[1, 2] #3[6]"
    );
}

#[test]
fn delete_6() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..8 {
        t.insert(val);
        debug!(t.format_debug());
    }

    assert_eq!(
        t.format_debug(),
        "#6[4]
#2[2] #5[6]
#0[1] #1[3] #3[5] #4[7]"
    );

    t.delete(7);

    assert_eq!(
        t.format_debug(),
        "#2[2, 4]
#0[1] #1[3] #3[5, 6]"
    );
}

#[test]
fn delete_7() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..4 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#2[2]
#0[1] #1[3]"
    );

    t.delete(2);
    assert_eq!(t.format_debug(), "#0[1, 3]");

    t.delete(1);
    assert_eq!(t.format_debug(), "#0[3]");

    t.delete(3);
    assert_eq!(t.format_debug(), "#0[]");
}

#[test]
fn delete_8() {
    let mut t = Tree::<_, 3>::default();
    for val in [4, 3, 2, 1] {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#2[3]
#0[1, 2] #1[4]"
    );
    t.delete(4);

    assert_eq!(
        t.format_debug(),
        "#2[2]
#0[1] #1[3]"
    );
}

#[test]
fn delete_9_rotate_left() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..10 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#6[4]
#2[2] #5[6, 8]
#0[1] #1[3] #3[5] #4[7] #7[9]"
    );

    t.delete(2);

    assert_eq!(
        t.format_debug(),
        "#6[6]
#2[4] #5[8]
#0[1, 3] #3[5] #4[7] #7[9]"
    );

    let root = &t.arena[t.root_id];
    let node4 = &t.arena[*root.children.first().unwrap()];
    assert_eq!(node4.children.len(), 2);
    let node8 = &t.arena[*root.children.last().unwrap()];
    assert_eq!(node8.children.len(), 2);
}

#[test]
fn delete_10_rotate_right() {
    let mut t = Tree::<_, 3>::default();
    let mut vals: Vec<usize> = (1..10).collect();
    vals.reverse();
    for val in vals {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#6[6]
#2[2, 4] #5[8]
#0[1] #7[3] #4[5] #3[7] #1[9]"
    );

    t.delete(8);

    assert_eq!(
        t.format_debug(),
        "#6[4]
#2[2] #5[6]
#0[1] #7[3] #4[5] #3[7, 9]"
    );

    let root = &t.arena[t.root_id];
    let node2 = &t.arena[*root.children.first().unwrap()];
    assert_eq!(node2.children.len(), 2);
    let node6 = &t.arena[*root.children.last().unwrap()];
    assert_eq!(node6.children.len(), 2);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rand_vec(n: usize, seed: usize) -> Vec<usize> {
        use rand::seq::SliceRandom;
        use rand::SeedableRng;
        use rand_pcg::Pcg64;

        let mut rng = Pcg64::seed_from_u64(seed as u64);
        let mut vec: Vec<_> = (0..n).collect();
        vec.shuffle(&mut rng);
        return vec;
    }

    #[test]
    fn delete_11_delete_all() {
        let vals = rand_vec(20, 2);

        let mut t = Tree::<_, 3>::default();
        for val in vals {
            t.insert(val);
        }

        let vals = rand_vec(20, 3);
        for val in vals {
            t.delete(val);
        }

        assert_eq!(t.format_debug(), "#0[]");
    }

    #[test]
    fn huge_insert_delete() {
        use std::time::Instant;

        let vals: Vec<usize> = (0..1_000_000).collect();

        let mut t = Tree::<_, 256>::default();
        let now = Instant::now();
        for &val in vals.iter() {
            t.insert(val)
        }
        println!("{}", now.elapsed().as_millis());

        let now = Instant::now();
        for &val in vals.iter() {
            t.delete(val);
        }
        println!("{}", now.elapsed().as_millis());
    }

    #[test]
    fn huge_insert() {
        use std::time::Instant;

        let vals: Vec<usize> = (0..1_000_000).collect();

        let mut t = Tree::<_, 256>::default();
        let now = Instant::now();
        for &val in vals.iter() {
            t.insert(val)
        }
        println!("{}", now.elapsed().as_millis());
    }

    #[test]
    fn huge_rand_insert() {
        use std::time::Instant;

        let vals = rand_vec(1_000_000, 0);

        let mut t = Tree::<_, 256>::default();
        let now = Instant::now();
        for &val in vals.iter() {
            t.insert(val)
        }
        println!("{}", now.elapsed().as_millis());
    }

    #[test]
    fn huge_rand_get() {
        use std::time::Instant;

        let vals = rand_vec(1_000_000, 0);

        let mut t = Tree::<_, 256>::default();
        let now = Instant::now();
        for &val in vals.iter() {
            t.insert(val)
        }
        println!("{}", now.elapsed().as_millis());

        let vals = rand_vec(1_000_000, 1);
        let now = Instant::now();
        for &val in vals.iter() {
            assert_eq!(t.get(val).is_some(), true);
        }
        println!("{}", now.elapsed().as_millis());

        assert_eq!(t.get(1_000_001).is_none(), true);
    }
}

#[test]
fn sibling() {
    let mut t = Tree::<_, 3>::default();
    for val in 1..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "#6[4]
#2[2] #5[6]
#0[1] #1[3] #3[5] #4[7]"
    );

    let root = &t.arena[t.root_id];
    let left_id = root.children.first().unwrap();
    let right_id = root.children.last().unwrap();
    let (left_left_id, left_idx, right_left_id) = t.sibling(*left_id);
    assert_eq!(left_left_id, None);
    assert_eq!(left_idx, Some(0));
    assert_eq!(right_left_id, Some(*right_id));

    let (left_right_id, right_idx, right_right_id) = t.sibling(*right_id);
    assert_eq!(left_right_id, Some(*left_id));
    assert_eq!(right_idx, Some(1));
    assert_eq!(right_right_id, None);

    let (node_id, value_idx, found) = t.search(3);
    assert_eq!(found, true);
    assert_eq!(value_idx, 0);
    let (left_node_id, node_idx, right_node_id) = t.sibling(node_id);
    assert_ne!(left_node_id, None);
    assert_eq!(t.arena[left_node_id.unwrap()].values.as_slice(), vec![1]);
    assert_eq!(node_idx, Some(1));
    assert_eq!(right_node_id, None);
}
