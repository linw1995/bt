use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct Node<T> {
    idx: usize,
    parent: Option<usize>,
    // TODO: change into fixed size structure
    vals: Vec<T>,
    children: Vec<usize>,
}

impl<T> Node<T> {
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct Tree<T> {
    root_id: usize,
    m: usize,
    arena: Vec<Node<T>>,
}

impl<T> Tree<T>
where
    T: Ord + Copy + Default + Debug,
{
    fn alloc_node(&mut self) -> usize {
        let idx = self.arena.len();
        let mut node = Node {
            idx,
            vals: Vec::with_capacity(self.m - 1),
            children: Vec::with_capacity(self.m),
            parent: None,
        };
        node.idx = idx;
        self.arena.push(node);
        idx
    }

    pub fn insert(&mut self, val: T) {
        match self.search(val) {
            None => {
                let id = self.alloc_node();
                let node = &mut self.arena[id];
                node.vals.push(val);
            }
            Some((mut cur_id, val_idx, found)) => {
                if found {
                    return;
                }

                self.arena[cur_id].vals.insert(val_idx, val);

                loop {
                    let cur = &self.arena[cur_id];
                    debug!(&cur);
                    if cur.vals.len() < self.m as usize {
                        return;
                    } else {
                        // alloc or reuse a parent node
                        let parent_id = match cur.parent {
                            None => {
                                let parent_id = self.alloc_node();
                                self.root_id = parent_id;
                                self.arena[parent_id].children.push(cur_id);
                                parent_id
                            }
                            Some(parent_id) => parent_id,
                        };

                        // split node
                        let (separator_val, right_vals, right_children) = {
                            let left = &mut self.arena[cur_id];
                            left.parent = Some(parent_id);

                            let lchildren = &mut left.children;
                            let rchildren = if !lchildren.is_empty() {
                                lchildren.split_off(self.m / 2 + 1)
                            } else {
                                vec![]
                            };

                            let vals = &mut left.vals.split_off(self.m / 2);
                            debug!(
                                vals[0],           // separator value
                                vals.split_off(1), // right values
                                rchildren,         // right children
                            )
                        };

                        // alloc a right node
                        let right_id = {
                            let right_id = self.alloc_node();
                            let right = &mut self.arena[right_id];
                            right.parent = Some(parent_id);
                            right.vals.extend(right_vals);
                            right.children.extend(right_children);
                            debug!(right_id)
                        };

                        // find where to insert the new right node in the parent as one of the children
                        {
                            let parent = &mut self.arena[parent_id];
                            let mut insert_idx = parent.vals.len();
                            for (idx, val) in parent.vals.iter().enumerate() {
                                if &separator_val < val {
                                    insert_idx = idx;
                                    break;
                                }
                            }
                            debug!(&parent, insert_idx);
                            parent.vals.insert(insert_idx, separator_val);
                            parent.children.insert(insert_idx + 1, right_id);
                        }

                        // Maybe the parent node needs to be rebalanced.
                        cur_id = parent_id;
                    }
                }
            }
        }
    }

    /// locate the (node_id, value_idx) for inserting the value.
    fn search(&self, val: T) -> Option<(usize, usize, bool)> {
        if self.arena.is_empty() {
            return None;
        }
        let mut cur = &self.arena[self.root_id];
        loop {
            debug!(cur);
            // find the insert index of value in the current node values
            let mut insert_idx = cur.vals.len();
            for (idx, &left) in cur.vals.iter().enumerate() {
                if left < val {
                    continue;
                }
                insert_idx = idx;
                if val == left {
                    // the val is found in current node vals
                    return Some((cur.idx, insert_idx, true));
                }
                break;
            }

            // try to find a space for inserting the value from the left sub-tree
            if cur.children.len() > insert_idx {
                cur = &self.arena[cur.children[insert_idx]];
                continue;
            }
            return Some((cur.idx, insert_idx, false));
        }
    }

    pub fn range(&self, _begin: T, _end: T) -> Vec<T> {
        todo!();
    }

    pub fn traversal_bfs(&self) -> Vec<T> {
        use std::collections::VecDeque;
        let mut q = VecDeque::with_capacity(self.arena.len());
        let mut cur = &self.arena[self.root_id];

        let mut path = Vec::new();
        loop {
            for &val in cur.vals.iter() {
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

    pub fn traversal_bfs_groupby_level(&self) -> Vec<Vec<T>> {
        use std::collections::VecDeque;
        let mut q = VecDeque::with_capacity(self.arena.len());
        let mut cur = &self.arena[self.root_id];

        let mut path = Vec::new();
        let mut depth = 0;
        loop {
            if depth >= path.len() {
                path.push(Vec::new());
            }
            for &val in cur.vals.iter() {
                path[depth].push(val);
            }
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
        path
    }
}

#[test]
fn insert_root() {
    let mut t = Tree::default();
    t.m = 3;
    t.insert(1);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![1]);
}

#[test]
fn insert_1() {
    let mut t = Tree::default();
    t.m = 3;
    t.insert(9);
    t.insert(10);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![9, 10]);

    let mut t = Tree::default();
    t.m = 3;
    t.insert(10);
    t.insert(9);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![9, 10]);
}

#[test]
fn insert_2() {
    let mut t = Tree::default();
    t.m = 4;
    t.insert(9);
    t.insert(11);
    t.insert(10);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![9, 10, 11]);
}

#[test]
fn insert_3() {
    let mut t = Tree::default();
    t.m = 3;
    t.insert(9);
    t.insert(10);
    t.insert(0);
    assert_eq!(t.arena.len(), 3);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![9]);
    assert_eq!(root.children, vec![0, 2]);
    assert_eq!(t.arena[0].vals, vec![0]);
    assert_eq!(t.arena[2].vals, vec![10]);
}

#[test]
fn insert_4() {
    let mut t = Tree::default();
    t.m = 3;
    for &val in vec![1, 2, 3].iter() {
        t.insert(val);
    }
    assert_eq!(t.root_id, 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![2]);
    assert_eq!(root.children, vec![0, 2]);

    t.insert(4);
    let right = &t.arena[2];
    assert_eq!(right.vals, vec![3, 4]);

    t.insert(5);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![2, 4]);
    let most_right = &t.arena[root.children[2]];
    assert_eq!(most_right.vals, vec![5]);

    t.insert(6);
    let root = &t.arena[t.root_id];
    let most_right = &t.arena[root.children[2]];
    assert_eq!(most_right.vals, vec![5, 6]);

    t.insert(7);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![4]);
    let left = &t.arena[root.children[0]];
    assert_eq!(left.vals, vec![2]);
    let right = &t.arena[root.children[1]];
    assert_eq!(right.vals, vec![6]);
}

#[test]
fn traversal_bfs() {
    let mut t = Tree::default();
    t.m = 3;
    for val in 1..8 {
        t.insert(val);
    }

    assert_eq!(t.traversal_bfs(), vec![4, 2, 6, 1, 3, 5, 7]);

    assert_eq!(
        t.traversal_bfs_groupby_level(),
        vec![vec![4], vec![2, 6], vec![1, 3, 5, 7]]
    );
}
