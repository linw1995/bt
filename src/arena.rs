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
    m: usize,
    arena: Vec<Node<T>>,
    root_id: usize,
}

impl<T> Tree<T>
where
    T: Ord + Copy + Default + Debug,
{
    pub fn new(m: usize) -> Self {
        let mut t = Tree::default();
        t.m = m;
        t.root_id = t.alloc_node();
        t
    }
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

    pub fn insert(&mut self, value: T) {
        match self.search(value) {
            (_, _, true) => {}
            (mut cur_id, val_idx, false) => {
                let cur = &mut self.arena[cur_id];
                cur.vals.insert(val_idx, value);

                // rebalance tree
                loop {
                    let cur = &self.arena[cur_id];
                    if cur.vals.len() < self.m as usize {
                        return;
                    } else {
                        // alloc or get a parent node
                        let parent_id = match cur.parent {
                            None => {
                                // The current node is the root node of the tree.
                                // Splitting it needs to create a new root node.
                                self.root_id = self.alloc_node();

                                let old_root = &mut self.arena[cur_id];
                                old_root.parent = Some(self.root_id);

                                let parent = &mut self.arena[self.root_id];
                                parent.children.push(cur_id);

                                self.root_id
                            }
                            Some(parent_id) => parent_id,
                        };

                        // split current node as left and right
                        let left_id = cur_id;
                        let (separator_val, right_vals, right_children) = {
                            let left = &mut self.arena[left_id];
                            // split the values of the left node
                            let vals = &mut left.vals.split_off(self.m / 2);

                            // split the children of the left node
                            let left_children = &mut left.children;
                            let right_children = if !left_children.is_empty() {
                                // the size of an internal node children is always larger than self.m / 2 + 1
                                left_children.split_off(self.m / 2 + 1)
                            } else {
                                // or it is just a leaf node with no child
                                vec![]
                            };

                            debug!(
                                vals[0],           // separator value
                                vals.split_off(1), // right values
                                right_children,    // right children
                            )
                        };

                        // alloc a right node
                        let right_id = {
                            let right_id = self.alloc_node();
                            {
                                let right = &mut self.arena[right_id];
                                right.parent = Some(parent_id);
                                right.vals.extend(right_vals);
                            }
                            // update the parent of children as the right node
                            for &child_id in right_children.iter() {
                                self.arena[child_id].parent = Some(right_id);
                            }
                            {
                                let right = &mut self.arena[right_id];
                                right.children.extend(right_children);
                            }
                            right_id
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
            node.vals.remove(value_idx);
        }
        let node = &self.arena[node_id];
        if node.is_leaf() {
            node_id
        } else {
            let (from_id, from_value_idx) = match self.adjacent_children(node_id, value_idx) {
                (Some(&left_id), None) => {
                    let (most_right_id, _) = self.most_right(left_id);
                    (most_right_id, self.arena[most_right_id].vals.len() - 1)
                }
                (Some(&left_id), Some(&right_id)) => {
                    let (most_right_id, most_right_depth) = self.most_right(left_id);
                    let (most_left_id, most_left_depth) = self.most_left(right_id);
                    if most_left_depth < most_right_depth {
                        (most_right_id, self.arena[most_right_id].vals.len() - 1)
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
                from_node.vals.remove(from_value_idx)
            };

            let node = &mut self.arena[node_id];
            node.vals.insert(value_idx, new_separator_value);

            from_id
        }
    }

    fn rebalance(&mut self, node_id: usize) {
        let mut cur_id = node_id;
        loop {
            let node = &self.arena[cur_id];
            if node.is_root() || node.vals.len() >= (self.m - 1) / 2 {
                return;
            }
            let (left, _, right) = self.sibling(cur_id);
            let right_len = if let Some(id) = right {
                self.arena[id].vals.len()
            } else {
                0
            };
            let left_len = if let Some(id) = left {
                self.arena[id].vals.len()
            } else {
                0
            };

            let (max_len, is_left_max) = if left_len > right_len {
                (left_len, true)
            } else {
                (right_len, false)
            };

            if max_len > (self.m - 1) / 2 {
                if is_left_max {
                    self.rotate_right(cur_id);
                } else {
                    self.rotate_left(cur_id);
                }
            } else if right_len > 0 && left_len > 0 {
                // Node merges a minor sibling node
                if is_left_max {
                    self.merge_right(cur_id);
                } else {
                    self.merge_left(cur_id);
                }
            } else if right_len > 0 {
                // left_len == 0
                cur_id = self.merge_right(cur_id);
            } else if left_len > 0 {
                // right_len == 0
                cur_id = self.merge_left(cur_id);
            } else {
                unreachable!();
            }
            let parent_id = self.arena[cur_id].parent.unwrap();
            let parent = &self.arena[parent_id];
            if parent.is_root() && parent.vals.is_empty() {
                self.arena[cur_id].parent = None;
                self.root_id = cur_id;
                return;
            }
            cur_id = parent_id;
        }
    }

    fn rotate_left(&mut self, node_id: usize) {
        let parent_id = self.arena[node_id].parent.unwrap();
        if let (_, Some(node_child_idx), Some(right_id)) = self.sibling(node_id) {
            let value_idx = node_child_idx;
            let separator = self.arena[parent_id].vals.remove(value_idx);
            self.arena[node_id].vals.push(separator);
            let new_separator = self.arena[right_id].vals.remove(0);
            self.arena[parent_id].vals.insert(value_idx, new_separator)
        } else {
            unreachable!()
        };
    }

    fn rotate_right(&mut self, node_id: usize) {
        let parent_id = self.arena[node_id].parent.unwrap();
        if let (Some(left_id), Some(node_child_idx), _) = self.sibling(node_id) {
            let value_idx = node_child_idx - 1;
            let separator = self.arena[parent_id].vals.remove(value_idx);
            self.arena[node_id].vals.push(separator);
            let new_separator = {
                let left = &mut self.arena[left_id];
                left.vals.remove(left.vals.len() - 1)
            };
            self.arena[parent_id].vals.insert(value_idx, new_separator)
        } else {
            unreachable!()
        };
    }

    fn merge_left(&mut self, node_id: usize) -> usize {
        if let (Some(left_id), Some(node_child_idx), _) = self.sibling(node_id) {
            self.merge_sibling_nodes(left_id, node_child_idx - 1, node_id);
            left_id
        } else {
            unreachable!()
        }
    }

    fn merge_right(&mut self, node_id: usize) -> usize {
        if let (_, Some(node_child_idx), Some(right_id)) = self.sibling(node_id) {
            self.merge_sibling_nodes(node_id, node_child_idx, right_id);
            node_id
        } else {
            unreachable!()
        }
    }

    fn merge_sibling_nodes(&mut self, node_id: usize, separator_idx: usize, right_id: usize) {
        let parent_id = self.arena[node_id].parent.unwrap();

        let parent = &mut self.arena[parent_id];
        let separator = parent.vals.remove(separator_idx);
        self.arena[node_id].vals.push(separator);

        let right_values = &mut self.arena[right_id].vals.split_off(0);
        self.arena[node_id].vals.append(right_values);
        let right_children = &mut self.arena[right_id].children.split_off(0);
        self.arena[node_id].children.append(right_children);

        self.arena[parent_id].children.remove(separator_idx + 1);
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
        if value_idx == 0 {
            (None, cur.children.first())
        } else if value_idx < cur.children.len() - 1 {
            (
                Some(&cur.children[value_idx - 1]),
                Some(&cur.children[value_idx]),
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

    /// locate the (node_id, value_idx) for inserting the value.
    fn search(&self, val: T) -> (usize, usize, bool) {
        let mut cur = &self.arena[self.root_id];
        loop {
            // find the insert index of value in the current node values
            let mut insert_idx = cur.vals.len();
            for (idx, &left) in cur.vals.iter().enumerate() {
                if left < val {
                    continue;
                }
                insert_idx = idx;
                if val == left {
                    // the value is found in the current node values
                    return (cur.idx, insert_idx, true);
                }
                break;
            }

            // try to find a space for inserting the value from the left sub-tree
            if cur.children.len() > insert_idx {
                cur = &self.arena[cur.children[insert_idx]];
                continue;
            }
            return (cur.idx, insert_idx, false);
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
            path[depth].push(&cur.vals);
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
            for n in row.iter() {
                rv.push_str(&format!("{:?} ", n));
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
    let mut t = Tree::new(3);
    t.insert(1);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![1]);
}

#[test]
fn insert_1() {
    let mut t = Tree::new(3);
    t.insert(9);
    t.insert(10);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![9, 10]);

    let mut t = Tree::new(3);
    t.insert(10);
    t.insert(9);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![9, 10]);
}

#[test]
fn insert_2() {
    let mut t = Tree::new(4);
    t.insert(9);
    t.insert(11);
    t.insert(10);
    assert_eq!(t.arena.len(), 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![9, 10, 11]);
}

#[test]
fn insert_3() {
    let mut t = Tree::new(3);
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
    let mut t = Tree::new(3);
    for &val in vec![1, 2, 3].iter() {
        t.insert(val);
    }
    assert_eq!(t.root_id, 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.parent, None);
    assert_eq!(root.vals, vec![2]);
    assert_eq!(root.children, vec![0, 2]);

    t.insert(4);
    let right = &t.arena[2];
    assert_eq!(right.parent, Some(1));
    assert_eq!(right.vals, vec![3, 4]);

    t.insert(5);
    assert_eq!(t.root_id, 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.parent, None);
    assert_eq!(root.vals, vec![2, 4]);
    let most_right = &t.arena[root.children[2]];
    assert_eq!(most_right.parent, Some(t.root_id));
    assert_eq!(most_right.vals, vec![5]);

    t.insert(6);
    assert_eq!(t.root_id, 1);
    let root = &t.arena[t.root_id];
    let most_right = &t.arena[root.children[2]];
    assert_eq!(most_right.parent, Some(t.root_id));
    assert_eq!(most_right.vals, vec![5, 6]);

    t.insert(7);
    assert_eq!(t.root_id, 5);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![4]);
    let left = &t.arena[root.children[0]];
    assert_eq!(left.parent, Some(t.root_id));
    assert_eq!(left.vals, vec![2]);
    let right = &t.arena[root.children[1]];
    assert_eq!(right.parent, Some(t.root_id));
    assert_eq!(right.vals, vec![6]);

    let most_left = &t.arena[left.children[0]];
    assert_eq!(most_left.parent, Some(left.idx));
    assert_eq!(most_left.vals, vec![1]);
    let most_right = &t.arena[right.children[1]];
    assert_eq!(most_right.parent, Some(right.idx));
    assert_eq!(most_right.vals, vec![7]);
}

#[test]
fn traversal_bfs() {
    let mut t = Tree::new(3);
    for val in 1..8 {
        t.insert(val);
    }

    assert_eq!(t.traversal_bfs(), vec![4, 2, 6, 1, 3, 5, 7]);
}

#[test]
fn format_debug_1() {
    let mut t = Tree::new(3);
    for val in 1..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[4]
[2] [6]
[1] [3] [5] [7]"
    );
}

#[test]
fn format_debug_2() {
    let mut t = Tree::new(3);
    for val in 1..7 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [5, 6]"
    );
}

#[test]
fn format_debug_3() {
    let mut t = Tree::new(3);
    for val in 1..6 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [5]"
    );
}

#[test]
fn format_debug_4() {
    let t = Tree::<usize>::default();
    assert_eq!(t.format_debug(), "[]");
}

#[test]
fn delete_notfound() {
    let mut t = Tree::new(3);
    for val in 1..4 {
        t.insert(val);
    }
    assert_eq!(t.delete(4), false);
}

#[test]
fn delete_1() {
    let mut t = Tree::new(3);
    for val in 1..5 {
        t.insert(val);
    }
    for val in 6..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [6, 7]"
    );

    t.delete(6);

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [7]"
    );
}

#[test]
fn delete_2() {
    let mut t = Tree::new(3);
    for val in 1..5 {
        t.insert(val);
    }
    for val in 6..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [6, 7]"
    );

    t.delete(4);

    assert_eq!(
        t.format_debug(),
        "[2, 6]
[1] [3] [7]"
    );
}

#[test]
fn delete_3() {
    let mut t = Tree::new(3);
    for val in 1..5 {
        t.insert(val);
    }
    for val in 6..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [6, 7]"
    );

    t.delete(3);

    assert_eq!(
        t.format_debug(),
        "[2, 6]
[1] [4] [7]"
    );
}

#[test]
fn delete_4() {
    let mut t = Tree::new(3);
    for val in 1..5 {
        t.insert(val);
    }
    for val in 6..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [6, 7]"
    );

    t.delete(1);

    assert_eq!(
        t.format_debug(),
        "[4]
[2, 3] [6, 7]"
    );
}

#[test]
fn delete_5() {
    let mut t = Tree::new(3);
    for val in 1..5 {
        t.insert(val);
    }
    t.insert(6);

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [6]"
    );

    t.delete(3);

    assert_eq!(
        t.format_debug(),
        "[4]
[1, 2] [6]"
    );
}

#[test]
fn delete_6() {
    let mut t = Tree::new(3);
    for val in 1..8 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[4]
[2] [6]
[1] [3] [5] [7]"
    );

    t.delete(7);

    assert_eq!(
        t.format_debug(),
        "[2, 4]
[1] [3] [5, 6]"
    );
}

#[test]
fn delete_7() {
    let mut t = Tree::new(3);
    for val in 1..4 {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[2]
[1] [3]"
    );

    t.delete(2);
    assert_eq!(t.format_debug(), "[1, 3]");

    t.delete(1);
    assert_eq!(t.format_debug(), "[3]");

    t.delete(3);
    assert_eq!(t.format_debug(), "[]");
}

#[test]
fn delete_8() {
    let mut t = Tree::new(3);
    for val in [4, 3, 2, 1] {
        t.insert(val);
    }

    assert_eq!(
        t.format_debug(),
        "[3]
[1, 2] [4]"
    );
    t.delete(4);

    assert_eq!(
        t.format_debug(),
        "[2]
[1] [3]"
    );
}
