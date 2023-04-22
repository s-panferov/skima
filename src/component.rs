use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::web::WebSys;
use crate::{Backend, Markup};

pub trait Component<B: Backend = WebSys>: Sized {
	type State: 'static = ();
	type Markup: Markup<B>;

	fn create(&self) -> Self::State;

	#[allow(unused)]
	fn update(&self, next: &Self, state: &mut Self::State) {}

	fn markup(self) -> ComponentMarkup<B, Self> {
		ComponentMarkup {
			component: self,
			_b: PhantomData,
		}
	}

	fn render(&self, state: &mut Self::State) -> Self::Markup;
}

pub struct ComponentMarkup<B: Backend, C: Component<B>> {
	component: C,
	_b: PhantomData<B>,
}

struct ComponentTreeStorage<B: Backend, C: Component<B>> {
	pub markup: RefCell<C::Markup>,
	pub state: RefCell<C::State>,
}

impl<B, C> Markup<B> for ComponentMarkup<B, C>
where
	C: Component<B> + 'static,
	B: Backend + 'static,
{
	fn has_own_node() -> bool {
		true
	}

	fn render(&self, tree: &crate::tree::Tree<B>) {
		let mut state = self.component.create();
		let markup = self.component.render(&mut state);
		markup.render(tree);
		tree.data_mut().set(Rc::new(ComponentTreeStorage::<B, C> {
			markup: RefCell::new(markup),
			state: RefCell::new(state),
		}));
	}

	fn diff(&self, prev: &Self, tree: &crate::tree::Tree<B>) {
		let storage = tree.data_mut().get::<Rc<ComponentTreeStorage<B, C>>>();
		let state = &mut storage.state.borrow_mut();
		C::update(&prev.component, &self.component, &mut *state);

		let markup = C::render(&self.component, &mut *state);
		markup.diff(&storage.markup.borrow_mut(), tree);
		storage.markup.replace(markup);
	}

	fn drop(&self, tree: &crate::tree::Tree<B>, should_unmount: bool) {
		let storage = tree.data_mut().remove::<Rc<ComponentTreeStorage<B, C>>>();
		storage.markup.borrow_mut().drop(tree, should_unmount);
	}
}
