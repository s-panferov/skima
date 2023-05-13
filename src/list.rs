use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use indexmap::IndexMap;

use crate::iter::IteratorExt;
use crate::tree::Tree;
use crate::{Backend, Markup};

struct ListData<K, M> {
	item_markup: HashMap<K, M>,
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
	state: RefCell<ListData<K, M>>,
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
		state: RefCell::new(ListData {
			item_markup: Default::default(),
		}),
		_b: PhantomData,
	}
}

impl<K, T, F, M, B> Markup<B> for List<K, T, F, M, B>
where
	K: Eq + std::hash::Hash + Clone + Debug,
	F: Fn(&T, &K) -> M,
	M: Markup<B>,
	B: Backend,
{
	fn has_own_node() -> bool {
		true
	}

	fn render(&mut self, parent: &crate::tree::Tree<B>) {
		let state = &mut self.state.borrow_mut();
		for item in self.data.iter() {
			let mut markup = (self.func)(item.1, item.0);
			let tree = Tree::new(parent);
			markup.render(&tree);
			state.item_markup.insert(item.0.clone(), markup);
		}
	}

	// TODO: do we need to implement key-based moves?
	fn diff(&mut self, prev: &mut Self, tree: &crate::tree::Tree<B>) {
		let mut prev_state = prev.state.borrow_mut();
		let mut next_state = self.state.borrow_mut();

		let prev_markup = &mut prev_state.item_markup;
		let next_markup = &mut next_state.item_markup;

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

			let prev_m = prev_markup.get_mut(prev_item.0).unwrap();
			let mut next_m = (self.func)(next_item.1, next_item.0);

			next_m.diff(prev_m, &tree);

			next_markup.insert(next_item.0.clone(), next_m);

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

			let prev_m = prev_markup.get_mut(prev_item.0).unwrap();
			let mut next_m = (self.func)(next_item.1, next_item.0);

			next_m.diff(prev_m, &tree);

			next_markup.insert(next_item.0.clone(), next_m);

			prev_range.end -= 1;
			next_range.end -= 1;
		}

		if prev_iter.peek().is_none() {
			// We only have new items, we need to add them
			for node in next_iter {
				let subtree = tree.insert_at(next_range.start);
				let mut markup = (self.func)(node.1, node.0);
				markup.render(&subtree);
				next_markup.insert(node.0.clone(), markup);
				next_range.start += 1;
			}

			return;
		} else if next_iter.peek().is_none() {
			// We only have old items, we need to remove them
			for prev_item in prev_iter {
				let subtree = tree.child_at(prev_range.start);

				let prev_m = prev_markup.get_mut(prev_item.0).unwrap();

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

					let prev_m = prev_markup.get_mut(prev_item.0).unwrap();

					prev_m.drop(&subtree, true);
					subtree.disconnect(true);

					// TODO: remove from the end instead, this version is super inefficient
					tree.remove_at(prev_range.start);
				}
				Some(next_item) => {
					let tree = tree.child_at(prev_range.start);

					let prev_m = prev_markup.get_mut(prev_item.0).unwrap();
					let mut next_m = (self.func)(next_item.1, next_item.0);

					next_m.diff(prev_m, &tree);

					next_markup.insert(next_item.0.clone(), next_m);

					prev_range.start += 1;
					next_range.start += 1;
				}
			}
		}

		for next_item in next_iter {
			let subtree = tree.insert_at(next_range.start);
			let mut markup = (self.func)(next_item.1, next_item.0);
			markup.render(&subtree);

			next_markup.insert(next_item.0.clone(), markup);

			next_range.start += 1;
			next_range.start += 1;
			// INSERTS
		}
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		let mut state = self.state.borrow_mut();
		let rendered = &mut state.item_markup;

		let mut cursor = None;
		for (i, key) in self.data.keys().enumerate() {
			if i == 0 {
				cursor = Some(tree.first_child());
			} else {
				cursor = Some(cursor.unwrap().next());
			}

			let markup = rendered.get_mut(key).unwrap();
			markup.drop(cursor.as_ref().unwrap(), should_unmount);
		}

		tree.clear()
	}
}
