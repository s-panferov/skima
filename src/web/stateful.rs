use std::any::Provider;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::context::{DefaultExt, DynInit, StatefulContext};
use crate::tree::Tree;
use crate::web::context::HasContext;
use crate::web::{Backend, Markup};

pub struct StatefulComponent<
	F: FnOnce(&mut StatefulContext<B, E>) -> M,
	M: Markup<B>,
	B: Backend,
	E,
> {
	factory: Option<F>,
	rendered: Option<M>,
	context: Option<StatefulContext<B, E>>,
	_b: PhantomData<B>,
}

pub fn stateful<F, M, B>(factory: F) -> impl Markup<B>
where
	F: FnOnce(&mut StatefulContext<B, DefaultExt<B>>) -> M,
	M: Markup<B>,
	B: Backend,
{
	StatefulComponent {
		factory: Some(factory),
		rendered: None,
		context: None,
		_b: PhantomData,
	}
}

impl<'a, F, M, B, E> Markup<B> for StatefulComponent<F, M, B, E>
where
	F: FnOnce(&mut StatefulContext<B, E>) -> M,
	M: Markup<B>,
	B: Backend + 'a,
	E: DynInit + 'a,
{
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn dynamic() -> bool {
		M::dynamic()
	}

	fn render(&mut self, tree: &Tree<B>) {
		struct P {}
		impl Provider for P {
			fn provide<'a>(&'a self, demand: &mut std::any::Demand<'a>) {}
		}

		struct W<B: Backend, E>(Rc<RefCell<StatefulContext<B, E>>>);

		impl<B: Backend, E> HasContext<B, E> for RefCell<StatefulContext<B, E>> {
			fn context(&self) -> std::cell::RefMut<StatefulContext<B, E>> {
				self.borrow_mut()
			}
		}

		let mut context = StatefulContext {
			tree: tree.clone(),
			ext: E::dyn_init(&P {}),
		};

		if let Some(factory) = self.factory.take() {
			let mut markup = factory(&mut context);
			markup.render(tree);
			self.rendered = Some(markup);
		} else if let Some(markup) = self.rendered.as_mut() {
			markup.render(tree);
		}

		self.context = Some(context);
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		if !Self::dynamic() {
			return;
		}

		let mut context = self.context.as_mut().unwrap();
		if let Some(factory) = self.factory.take() {
			let mut markup = factory(context);
			markup.diff(prev.rendered.as_mut().unwrap(), tree);
			self.rendered = Some(markup);
		} else if let Some(markup) = self.rendered.as_mut() {
			markup.diff(prev.rendered.as_mut().unwrap(), tree);
		}
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		if let Some(markup) = self.rendered.as_mut() {
			markup.drop(tree, should_unmount);
		}

		if Self::has_own_node() {
			tree.clear();
		}
	}
}
