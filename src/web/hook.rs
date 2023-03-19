use std::any::TypeId;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::tree::Tree;
use crate::{Backend, Markup};

pub struct WithTree<M: Markup<B>, B: Backend, F: Fn(&Tree<B>) -> M> {
	func: F,
	_b: PhantomData<B>,
}

pub fn hook<M, B, F>(func: F) -> impl Markup<B>
where
	M: Markup<B> + 'static,
	B: Backend,
	F: Fn(&Tree<B>) -> M + 'static,
{
	WithTree {
		func,
		_b: PhantomData,
	}
}

impl<M, B, F> Markup<B> for WithTree<M, B, F>
where
	M: Markup<B> + 'static,
	B: Backend,
	F: Fn(&Tree<B>) -> M + 'static,
{
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn render(&self, tree: &Tree<B>) {
		let markup = (self.func)(tree);
		markup.render(tree);
		tree.data_mut()
			.set_with_key::<Rc<M>>(fxhash::hash64(&TypeId::of::<F>()), Rc::new(markup));
	}

	fn diff(&self, _prev: &Self, tree: &Tree<B>) {
		let markup = (self.func)(tree);

		// TODO: get/set -> single op
		let prev = tree
			.data()
			.get_with_key::<Rc<M>>(fxhash::hash64(&TypeId::of::<F>()));
		markup.diff(&prev, tree);
		tree.data_mut()
			.set_with_key::<Rc<M>>(fxhash::hash64(&TypeId::of::<F>()), Rc::new(markup));
	}

	fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
		let markup = tree
			.data()
			.get_with_key::<Rc<M>>(fxhash::hash64(&TypeId::of::<F>()));

		markup.drop(tree, should_unmount)
	}
}
