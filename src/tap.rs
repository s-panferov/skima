use std::cell::RefCell;
use std::marker::PhantomData;

use crate::tree::Tree;
use crate::{Backend, Markup};

pub struct WithTree<M: Markup<B>, B: Backend, F: Fn(&Tree<B>) -> M> {
	func: F,
	state: RefCell<Option<M>>,
	_b: PhantomData<B>,
}

pub fn tap<M, B, F>(func: F) -> impl Markup<B>
where
	M: Markup<B> + 'static,
	B: Backend,
	F: Fn(&Tree<B>) -> M + 'static,
{
	WithTree {
		func,
		state: RefCell::new(None),
		_b: PhantomData,
	}
}

impl<M, B, F> Markup<B> for WithTree<M, B, F>
where
	M: Markup<B>,
	B: Backend,
	F: Fn(&Tree<B>) -> M,
{
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn render(&self, tree: &Tree<B>) {
		let markup = (self.func)(tree);
		markup.render(tree);
		self.state.replace(Some(markup));
	}

	fn diff(&self, _prev: &Self, tree: &Tree<B>) {
		let markup = (self.func)(tree);

		let mut prev_state = _prev.state.borrow_mut();
		let prev = prev_state.as_mut().unwrap();

		// TODO: get/set -> single op
		markup.diff(&prev, tree);
		self.state.replace(Some(markup));
	}

	fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
		self.state
			.borrow_mut()
			.as_mut()
			.unwrap()
			.drop(tree, should_unmount)
	}
}
