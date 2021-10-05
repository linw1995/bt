#[derive(Debug, Default)]
pub struct Node<T> {
	idx: usize,
	parent: Option<usize>,
	vals: Vec<T>,
	children: Vec<usize>,
}

impl<T> Node<T> {
	pub fn is_root(&self) -> bool {
		self.parent.is_none()
	}

	pub fn is_leaf(&self) -> bool {
		self.children.len() == 0
	}
}

#[derive(Debug, Default)]
pub struct Tree<T> {
	root_id: usize,
	m: u8,
	arena: Vec<Node<T>>,
}

impl<T> Tree<T>
where
	T: Ord + Copy + Default,
{
	fn alloc_node(&mut self) -> usize {
		let idx = self.arena.len();
		let mut node = Node::<T>::default();
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
			Some((cur_id, parent_id, val_idx, found)) => {}
		}
	}

	fn search(&self, val: T) -> Option<(usize, Option<usize>, i8, bool)> {
		if self.arena.len() == 0 {
			return None;
		}
		let mut cur = &self.arena[self.root_id];
		'tree: loop {
			match cur.vals.first() {
				None => return Some((cur.idx, cur.parent, -1, false)),
				Some(&begin) => {
					if val < begin {
						cur = &self.arena[cur.children[0]];
						continue;
					}
				}
			}
			if &val < cur.vals.first().unwrap() {
				cur = &self.arena[cur.children[0]];
			}
			let mut iter = cur.vals.windows(2);
			let mut idx = 1;
			loop {
				match iter.next() {
					Some(&[begin, end]) => {
						if begin == val {
							return Some((
								cur.idx, cur.parent, idx, true,
							));
						}
						if begin < val && val < end {
							cur = &self.arena[idx as usize];
							continue 'tree;
						}
					}
					None => break,
					_ => unreachable!(),
				}
				idx += 1;
			}
			if &val > cur.vals.last().unwrap() {
				cur = &self.arena[*cur.children.last().unwrap()];
				continue;
			}

			return Some((cur.idx, cur.parent, cur.vals.len() as i8, false));
		}
	}

	pub fn range(&self, begin: T, end: T) -> Vec<T> {
		todo!();
	}
}

#[test]
fn insert_root() {
	let mut t = Tree::default();
	t.insert(1);
	assert_eq!(t.arena.len(), 1);
	let root = &t.arena[t.root_id];
	assert_eq!(root.vals, vec![1]);
}
