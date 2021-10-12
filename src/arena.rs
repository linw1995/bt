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
                #[cfg(debug_assertions)]
                println!(
                    "search {:?} cur_id={:?} val_idx={:?} found={:?}",
                    val, cur_id, val_idx, found
                );

                if found {
                    return;
                }

                let cur = &mut self.arena[cur_id];

                #[cfg(debug_assertions)]
                println!("cur={:?} vals_len={:?}", cur, cur.vals.len());

                cur.vals.insert(val_idx, val);

                loop {
                    let cur = &self.arena[cur_id];

                    #[cfg(debug_assertions)]
                    println!("cur={:?} vals_len={:?}", cur, cur.vals.len());

                    if cur.vals.len() < self.m as usize {
                        return;
                    } else {
                        let parent_id = match cur.parent {
                            None => {
                                let parent_id = self.alloc_node();
                                self.root_id = parent_id;
                                let parent = &mut self.arena[parent_id];
                                parent.children.push(cur_id);
                                parent_id
                            }
                            Some(parent_id) => parent_id,
                        };

                        let (parent_val, right_vals, right_children) = {
                            let left = &mut self.arena[cur_id];
                            let left_vals = &mut left.vals;
                            let left_children = &mut left.children;

                            let vals = &mut left_vals.split_off(self.m / 2);
                            let right_vals = vals.split_off(1);
                            let parent_val = vals[0];

                            left.parent = Some(parent_id);
                            left.vals = left_vals.to_vec();

                            let mut right_children = vec![];
                            if !left_children.is_empty() {
                                right_children = left_children.split_off(self.m / 2 + 1);
                                left.children = left_children.to_vec();
                            }

                            (parent_val, right_vals, right_children)
                        };

                        let right_id = self.alloc_node();
                        let right = &mut self.arena[right_id];
                        right.parent = Some(parent_id);
                        right.vals.extend(right_vals);
                        right.children.extend(right_children);

                        let parent = &mut self.arena[parent_id];
                        let mut insert_idx = parent.vals.len();
                        for (idx, val) in parent.vals.iter().enumerate() {
                            if &parent_val < val {
                                insert_idx = idx;
                                break;
                            }
                        }

                        #[cfg(debug_assertions)]
                        println!("insert_idx={:?} parent={:?}", insert_idx, parent);

                        parent.vals.insert(insert_idx, parent_val);
                        parent.children.insert(insert_idx + 1, right_id);

                        if parent.vals.len() < self.m - 1 {
                            return;
                        } else {
                            cur_id = parent_id;
                        }
                    }
                }
            }
        }
    }

    fn search(&self, val: T) -> Option<(usize, usize, bool)> {
        if self.arena.is_empty() {
            return None;
        }
        let mut cur = &self.arena[self.root_id];
        loop {
            let mut insert_idx = cur.vals.len();
            let mut found = false;
            for (idx, &begin) in cur.vals.iter().enumerate() {
                if val > begin {
                    continue;
                }
                insert_idx = idx;
                if val == begin {
                    found = true
                }
                break;
            }
            if cur.children.len() > insert_idx {
                cur = &self.arena[cur.children[insert_idx]];
                continue;
            }
            return Some((cur.idx, insert_idx, found));
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
    println!("t={:?}", t);
    assert_eq!(t.root_id, 1);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![2]);
    assert_eq!(root.children, vec![0, 2]);

    t.insert(4);
    println!("t={:?}", t);
    let right = &t.arena[2];
    assert_eq!(right.vals, vec![3, 4]);

    t.insert(5);
    println!("t={:?}", t);
    let root = &t.arena[t.root_id];
    assert_eq!(root.vals, vec![2, 4]);
    let most_right = &t.arena[root.children[2]];
    assert_eq!(most_right.vals, vec![5]);

    t.insert(6);
    println!("t={:?}", t);
    let root = &t.arena[t.root_id];
    let most_right = &t.arena[root.children[2]];
    assert_eq!(most_right.vals, vec![5, 6]);

    t.insert(7);
    println!("t={:?}", t);
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
