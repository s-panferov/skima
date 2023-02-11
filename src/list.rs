use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::iter::IteratorExt;
use crate::tree::Tree;
use crate::{Backend, Markup};

pub struct ListData<K, M> {
	rendered: RefCell<HashMap<K, M>>,
}

pub struct List<K, T, F, M, B>
where
	K: std::hash::Hash + Eq,
	F: Fn(&T, &K) -> M,
	M: Markup<B>,
	B: Backend,
{
	data: IndexMap<K, T>,
	func: F,
	_b: PhantomData<B>,
}

pub fn list<'a, K, T, F, M, B>(
	iter: impl IntoIterator<Item = (K, T)>,
	func: F,
) -> List<K, T, F, M, B>
where
	K: Eq + std::hash::Hash,
	F: Fn(&T, &K) -> M,
	M: Markup<B>,
	B: Backend,
{
	let mut set = IndexMap::new();
	for a in iter.into_iter() {
		set.insert(a.0, a.1);
	}

	List {
		data: set,
		func,
		_b: PhantomData,
	}
}

impl<K, T, F, M, B> Markup<B> for List<K, T, F, M, B>
where
	K: Eq + std::hash::Hash + Clone + Debug + 'static,
	F: Fn(&T, &K) -> M,
	M: Markup<B> + 'static,
	B: Backend,
{
	fn has_own_node() -> bool {
		true
	}

	fn render(&self, parent: &crate::tree::Tree<B>) {
		let data = Rc::new(ListData {
			rendered: Default::default(),
		});

		let mut rendered = data.rendered.borrow_mut();
		for item in self.data.iter() {
			let markup = (self.func)(&item.1, &item.0);
			let tree = Tree::new(&parent);
			markup.render(&tree);
			rendered.insert(item.0.clone(), markup);
		}

		std::mem::drop(rendered);
		parent.data_mut().set(data);
	}

	// TODO: implement key-based moves
	fn diff(&self, prev: &Self, tree: &crate::tree::Tree<B>) {
		let data = tree.data().get::<Rc<ListData<K, M>>>();
		let mut rendered = data.rendered.borrow_mut();

		let mut next_iter = self.data.iter().de_peekable();
		let mut prev_iter = prev.data.iter().de_peekable();

		let mut prev_range = 0..prev.data.len();
		let mut next_range = 0..self.data.len();

		// Sync same items at the beginning
		while prev_iter
			.peek()
			.and_then(|kprev| next_iter.peek().filter(|knext| kprev.0 == knext.0))
			.is_some()
		{
			let prev_item = prev_iter.next().unwrap();
			let next_item = next_iter.next().unwrap();
			let tree = tree.child_at(prev_range.start);

			let prev_m = rendered.get(&prev_item.0).unwrap();
			let next_m = (self.func)(next_item.1, next_item.0);

			next_m.diff(prev_m, &tree);

			rendered.insert(next_item.0.clone(), next_m);

			prev_range.start += 1;
			next_range.start += 1;
		}

		// Sync same items at the end
		while prev_iter
			.peek_back()
			.and_then(|kprev| next_iter.peek_back().filter(|knext| kprev.0 == knext.0))
			.is_some()
		{
			let prev_item = prev_iter.next_back().unwrap();
			let next_item = next_iter.next_back().unwrap();

			let tree = tree.child_at(prev_range.start);

			let prev_m = rendered.get(&prev_item.0).unwrap();
			let next_m = (self.func)(next_item.1, next_item.0);

			next_m.diff(prev_m, &tree);

			rendered.insert(next_item.0.clone(), next_m);

			prev_range.end -= 1;
			next_range.end -= 1;
		}

		if prev_iter.peek().is_none() {
			// We only have new items, we need to add them
			for node in next_iter {
				let subtree = tree.insert_at(next_range.start);
				let markup = (self.func)(node.1, node.0);
				markup.render(&subtree);
				rendered.insert(node.0.clone(), markup);
				next_range.start += 1;
			}
			return;
		} else if next_iter.peek().is_none() {
			// We only have old items, we need to remove them
			for prev_item in prev_iter {
				let subtree = tree.child_at(prev_range.start);

				let prev_m = rendered.remove(&prev_item.0).unwrap();

				prev_m.drop(&subtree, true);
				subtree.disconnect(true);

				// TODO: remove from the end instead, this version is super inefficient
				tree.remove_at(prev_range.start);
			}

			return;
		}

		for prev_item in prev_iter {
			match next_iter.next() {
				None => {
					let subtree = tree.child_at(prev_range.start);
					let prev_m = rendered.remove(&prev_item.0).unwrap();

					prev_m.drop(&subtree, true);
					subtree.disconnect(true);

					// TODO: remove from the end instead, this version is super inefficient
					tree.remove_at(prev_range.start);
				}
				Some(next_item) => {
					let tree = tree.child_at(prev_range.start);

					let prev_m = rendered.get(&prev_item.0).unwrap();
					let next_m = (self.func)(next_item.1, next_item.0);

					next_m.diff(prev_m, &tree);

					rendered.insert(next_item.0.clone(), next_m);

					prev_range.start += 1;
					next_range.start += 1;
				}
			}
		}

		for next_item in next_iter {
			let subtree = tree.insert_at(next_range.start);
			let markup = (self.func)(next_item.1, next_item.0);
			markup.render(&subtree);

			rendered.insert(next_item.0.clone(), markup);

			next_range.start += 1;
			next_range.start += 1;
			// INSERTS
		}
	}

	fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
		let data = tree.data_mut().remove::<Rc<ListData<K, M>>>();
		let rendered = data.rendered.borrow_mut();

		let mut cursor = None;
		for (i, key) in self.data.keys().enumerate() {
			if i == 0 {
				cursor = Some(tree.first_child());
			} else {
				cursor = Some(cursor.unwrap().next());
			}

			let markup = rendered.get(key).unwrap();
			markup.drop(cursor.as_ref().unwrap(), should_unmount);
		}

		tree.clear()
	}
}
