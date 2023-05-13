use std::cell::RefCell;
use std::marker::PhantomData;

use crate::anydata::Envelope;
use crate::tree::Tree;
use crate::web::{Backend, Markup};

pub struct ConstantContext<B: Backend> {
	tree: Tree<B>,
}

impl<B: Backend> ConstantContext<B> {
	pub fn with_memo<T: Envelope + 'static>(&mut self, func: impl FnOnce() -> T) -> T::Output {
		if let Some(item) = self.tree.data().try_get::<T>() {
			item
		} else {
			let t = func();
			self.tree.data_mut().set(t);
			self.tree.data().get::<T>()
		}
	}
}

pub struct ConstantComponentFactory<
	F: FnOnce(&mut ConstantContext<B>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
> {
	factory: RefCell<Option<F>>,
	rendered: RefCell<Option<M>>,
	_b: PhantomData<B>,
}

pub fn constant<F, M, B>(factory: F) -> impl Markup<B>
where
	F: FnOnce(&mut ConstantContext<B>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
{
	ConstantComponentFactory {
		factory: RefCell::new(Some(factory)),
		rendered: RefCell::new(None),
		_b: PhantomData,
	}
}

impl<F, M, B> Markup<B> for ConstantComponentFactory<F, M, B>
where
	F: FnOnce(&mut ConstantContext<B>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
{
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn dynamic() -> bool {
		M::dynamic()
	}

	fn render(&mut self, tree: &Tree<B>) {
		let mut context = ConstantContext { tree: tree.clone() };

		if let Some(factory) = self.factory.take() {
			let mut markup = factory(&mut context);
			markup.render(tree);
			self.rendered.replace(Some(markup));
		} else if let Some(markup) = self.rendered.borrow_mut().as_mut() {
			markup.render(tree);
		}
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		if !Self::dynamic() {
			return;
		}

		let mut context = ConstantContext { tree: tree.clone() };
		if let Some(factory) = self.factory.take() {
			let mut markup = factory(&mut context);
			markup.diff(prev.rendered.borrow_mut().as_mut().unwrap(), tree);
			self.rendered.replace(Some(markup));
		} else if let Some(markup) = self.rendered.borrow_mut().as_mut() {
			markup.diff(prev.rendered.borrow_mut().as_mut().unwrap(), tree);
		}
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		if let Some(markup) = self.rendered.borrow_mut().as_mut() {
			markup.drop(tree, should_unmount);
		}

		if Self::has_own_node() {
			tree.clear();
		}
	}
}
