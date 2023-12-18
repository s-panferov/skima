use std::cell::RefCell;
use std::fmt::Display;
use std::marker::PhantomData;
use std::rc::Rc;

use super::context::{DefaultExt, DynInit, MaybeExtension, StatefulContext, WithEffects};
use crate::console_log;
use crate::tree::Tree;
use crate::web::context::HasContext;
use crate::web::{Backend, Markup};

pub struct StatefulComponent<
	F: FnOnce(&mut StatefulContext<B, E>) -> M,
	M: Markup<B>,
	B: Backend + 'static,
	E: MaybeExtension<WithEffects<B, E>> + 'static,
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
	B: Backend + 'static,
{
	StatefulComponent {
		factory: Some(factory),
		rendered: None,
		context: None,
		_b: PhantomData,
	}
}

impl<F, M, B, E> Drop for StatefulComponent<F, M, B, E>
where
	F: FnOnce(&mut StatefulContext<B, E>) -> M,
	M: Markup<B>,
	B: Backend + 'static,
	E: MaybeExtension<WithEffects<B, E>> + 'static,
{
	fn drop(&mut self) {
		tracing::info!("Reactive component destroyed");
	}
}

impl<'a, F, M, B, E> Markup<B> for StatefulComponent<F, M, B, E>
where
	F: FnOnce(&mut StatefulContext<B, E>) -> M,
	M: Markup<B>,
	B: Backend + 'a,
	E: DynInit + 'a,
	E: MaybeExtension<WithEffects<B, E>>,
{
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn dynamic() -> bool {
		M::dynamic()
	}

	fn render(&mut self, tree: &Tree<B>) {
		#[derive(Debug)]
		struct P {}

		impl Display for P {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				unreachable!()
			}
		}

		impl std::error::Error for P {
			fn provide<'a>(&'a self, req: &mut std::error::Request<'a>) {}
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

		if let Some(with_arena @ WithEffects { .. }) = context.ext.try_get() {
			with_arena.run_effects(&context);
		}

		console_log!("Stateful::render {:#?}", tree);

		self.context = Some(context);
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		console_log!("Stateful::diff {:#?}", tree);

		// take context from the prev version
		std::mem::swap(&mut self.context, &mut prev.context);

		if !Self::dynamic() {
			return;
		}

		let context = self.context.as_mut().unwrap();

		if let Some(with_arena @ WithEffects { .. }) = context.ext.try_get() {
			with_arena.reset_effects_alive();
		}

		if let Some(factory) = self.factory.take() {
			let mut markup = factory(context);
			markup.diff(prev.rendered.as_mut().unwrap(), tree);
			self.rendered = Some(markup);
		} else if let Some(markup) = self.rendered.as_mut() {
			markup.diff(prev.rendered.as_mut().unwrap(), tree);
		}

		if let Some(with_arena @ WithEffects { .. }) = context.ext.try_get() {
			with_arena.run_effects(context);
		}
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		console_log!("Stateful::drop {:#?}", tree);

		if let Some(markup) = self.rendered.as_mut() {
			markup.drop(tree, should_unmount);
		}

		let context = self.context.as_ref().unwrap();
		if let Some(effects @ WithEffects { .. }) = context.ext.try_get() {
			effects.cleanup_effects_internal(context)
		}

		if Self::has_own_node() {
			tree.clear();
		}
	}
}
