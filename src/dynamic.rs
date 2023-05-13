use std::marker::PhantomData;

use crate::tree::Tree;
use crate::{Backend, Markup};

pub struct Dynamic<M: Markup<B>, B: Backend> {
	markup: M,
	_b: PhantomData<B>,
}

pub fn dynamic<M: Markup<B>, B: Backend>(markup: M) -> Dynamic<M, B> {
	Dynamic {
		markup,
		_b: PhantomData,
	}
}

impl<M: Markup<B>, B: Backend> Markup<B> for Dynamic<M, B> {
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn dynamic() -> bool {
		true
	}

	fn render(&mut self, tree: &Tree<B>) {
		self.markup.render(tree);
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		self.markup.diff(&mut prev.markup, tree)
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		self.markup.drop(tree, should_unmount)
	}
}
