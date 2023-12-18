use std::borrow::Cow;
use std::marker::PhantomData;

use crate::tree::Tree;
use crate::{Backend, Markup};
pub struct Debug<M: Markup<B>, B: Backend> {
	name: &'static str,
	markup: M,
	_b: PhantomData<B>,
}

impl<M: Markup<B>, B: Backend> Markup<B> for Debug<M, B> {
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn dynamic() -> bool {
		M::dynamic()
	}

	fn render(&mut self, tree: &Tree<B>) {
		tree.name.replace(Cow::Borrowed(self.name));
		self.markup.render(tree);
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		self.markup.diff(&mut prev.markup, tree);
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		self.markup.drop(tree, should_unmount);
	}
}

pub fn debug<B: Backend, M: Markup<B>>(name: &'static str) -> impl Fn(M) -> Debug<M, B> {
	move |markup: M| Debug {
		name,
		markup,
		_b: PhantomData,
	}
}
