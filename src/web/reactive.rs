use std::any::{request_value, type_name, Any, Provider, TypeId};
use std::cell::{Cell, RefCell, RefMut};
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use observe::{Dependencies, Derived, Evaluation, Invalid, State, CHANGED};
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::JsValue;

use super::context::{
	DynInit, Extension, HasContext, MaybeExtension, StatefulContext, WithArena, WithCycle,
	WithEffects, WithMemo, WithState,
};
use super::WebSys;
use crate::reference::Mutable;
use crate::tree::Tree;
use crate::web::{Backend, Markup};

pub struct WithReactions {
	pub(crate) evaluation: Option<Evaluation>,
	pub(crate) dependencies: Dependencies,
	pub(crate) derived: Weak<dyn Derived>,
}

pub type ReactiveContext<B = WebSys, E = ReactiveExt<B>> = StatefulContext<B, E>;

impl<BACKEND: Backend, E> AsRef<Evaluation> for StatefulContext<BACKEND, E>
where
	E: Extension<WithReactions>,
{
	fn as_ref(&self) -> &Evaluation {
		let with_reactions = self.ext.get();
		with_reactions.evaluation.as_ref().unwrap()
	}
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_name = queueMicrotask)]
	pub fn queue_microtask(closure: &JsValue);
}

pub fn queue<F: FnOnce() + 'static>(func: F) {
	queue_microtask(&Closure::once_into_js(func));
}

pub trait IntoMemo {
	fn into_memo(&self) -> Rc<dyn Any>;
	fn from_memo(rc: Rc<dyn Any>) -> Self;
}

impl<T: Any> IntoMemo for Rc<T> {
	fn from_memo(rc: Rc<dyn Any>) -> Self {
		Rc::downcast(rc).map_err(|_| ()).unwrap()
	}

	fn into_memo(&self) -> Rc<dyn Any> {
		self.clone()
	}
}

impl<B: Backend, E> StatefulContext<B, E> {
	pub fn get<T: Any>(&self) -> &T
	where
		E: Extension<WithState>,
	{
		let state: &WithState = self.ext.get();
		state
			.state
			.get(&TypeId::of::<T>())
			// FIXME: remove format
			.unwrap_or_else(|| panic!("Item of type {} is now available", type_name::<T>()))
			.downcast_ref::<T>()
			.unwrap()
	}

	pub fn mutable<T: 'static>(&mut self, value: T) -> Mutable<T>
	where
		E: Extension<WithState>,
	{
		let state: &mut WithState = self.ext.get_mut();
		// TODO: Store mutables separately to avoid double allocation
		state
			.state
			.entry(TypeId::of::<Mutable<T>>())
			.or_insert_with(|| Box::new(Mutable::new(value)))
			.as_ref()
			.downcast_ref::<Mutable<T>>()
			.unwrap()
			.clone()
	}

	pub fn with<T: Any>(&mut self, value: T)
	where
		E: Extension<WithState>,
	{
		let state: &mut WithState = self.ext.get_mut();

		let tid = TypeId::of::<T>();
		state
			.state
			.entry(tid)
			.or_insert_with(|| Box::new(value) as Box<dyn Any>);
	}

	pub fn with_fn<T: Any>(&mut self, func: impl FnOnce() -> T)
	where
		E: Extension<WithState>,
	{
		let state: &mut WithState = self.ext.get_mut();

		let tid = TypeId::of::<T>();
		state
			.state
			.entry(tid)
			.or_insert_with(|| Box::new((func)()) as Box<dyn Any>);
	}

	pub fn update<T: Any>(&mut self, func: impl FnOnce(&mut T))
	where
		E: Extension<WithState>,
		E: MaybeExtension<WithReactions>,
	{
		let state: &mut WithState = self.ext.get_mut();
		state
			.state
			.entry(TypeId::of::<T>())
			.and_modify(|v| func(v.downcast_mut().unwrap()));

		if MaybeExtension::<WithReactions>::has(&self.ext) {
			observe::batch_microtask(|| {
				let reactions = self.ext.try_get_mut().unwrap();
				reactions
					.derived
					.upgrade()
					.unwrap()
					.invalidate(Invalid::Definitely);
			});
		}
	}

	pub fn set<T: Any>(&mut self, value: T)
	where
		E: Extension<WithState>,
		E: MaybeExtension<WithReactions>,
	{
		let state: &mut WithState = self.ext.get_mut();
		state
			.state
			.insert(TypeId::of::<T>(), Box::new(value) as Box<dyn Any>);

		if MaybeExtension::<WithReactions>::has(&self.ext) {
			observe::batch_microtask(|| {
				let reactions = self.ext.try_get_mut().unwrap();
				reactions
					.derived
					.upgrade()
					.unwrap()
					.invalidate(Invalid::Definitely);
			});
		}
	}
}

pub struct ReactiveComponentInner<
	F: Fn(&mut StatefulContext<B, E>) -> M,
	M: Markup<B>,
	B: Backend,
	E,
> {
	context: StatefulContext<B, E>,
	markup: M,
	factory: Rc<F>,
}

pub struct ReactiveComponentFactory<
	F: Fn(&mut StatefulContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E = (),
> {
	factory: Rc<F>,
	_b: PhantomData<B>,
	_m: PhantomData<M>,
	_e: PhantomData<E>,
}

pub struct ReactiveExt<B: 'static> {
	pub effects: WithEffects<B, Self>,
	pub state: WithState,
	pub memo: WithMemo,
	pub cycle: WithCycle<B, Self>,
	reactions: WithReactions,
}

impl<B> DynInit for ReactiveExt<B> {
	fn dyn_init(provider: &dyn Provider) -> Self {
		Self {
			effects: Default::default(),
			state: Default::default(),
			memo: Default::default(),
			cycle: WithCycle {
				this: request_value(provider).unwrap(),
			},
			reactions: WithReactions {
				evaluation: Default::default(),
				dependencies: Default::default(),
				derived: request_value(provider).unwrap(),
			},
		}
	}
}

impl<B> MaybeExtension<WithReactions> for ReactiveExt<B> {
	fn try_get(&self) -> Option<&WithReactions> {
		Some(&self.reactions)
	}

	fn try_get_mut(&mut self) -> Option<&mut WithReactions> {
		Some(&mut self.reactions)
	}
}

impl<B> MaybeExtension<WithEffects<B, Self>> for ReactiveExt<B> {
	fn try_get(&self) -> Option<&WithEffects<B, Self>> {
		Some(&self.effects)
	}

	fn try_get_mut(&mut self) -> Option<&mut WithEffects<B, Self>> {
		Some(&mut self.effects)
	}
}

impl<B> MaybeExtension<WithArena> for ReactiveExt<B> {
	fn try_get(&self) -> Option<&WithArena> {
		None
	}

	fn try_get_mut(&mut self) -> Option<&mut WithArena> {
		None
	}
}

impl<B> Extension<WithReactions> for ReactiveExt<B> {
	fn get(&self) -> &WithReactions {
		&self.reactions
	}
	fn get_mut(&mut self) -> &mut WithReactions {
		&mut self.reactions
	}
}

impl<B> Extension<WithCycle<B, Self>> for ReactiveExt<B> {
	fn get(&self) -> &WithCycle<B, Self> {
		&self.cycle
	}
	fn get_mut(&mut self) -> &mut WithCycle<B, Self> {
		&mut self.cycle
	}
}

impl<B> Extension<WithMemo> for ReactiveExt<B> {
	fn get(&self) -> &WithMemo {
		&self.memo
	}
	fn get_mut(&mut self) -> &mut WithMemo {
		&mut self.memo
	}
}

impl<B> Extension<WithState> for ReactiveExt<B> {
	fn get(&self) -> &WithState {
		&self.state
	}
	fn get_mut(&mut self) -> &mut WithState {
		&mut self.state
	}
}

impl<B> Extension<WithEffects<B, ReactiveExt<B>>> for ReactiveExt<B> {
	fn get(&self) -> &WithEffects<B, ReactiveExt<B>> {
		&self.effects
	}
	fn get_mut(&mut self) -> &mut WithEffects<B, ReactiveExt<B>> {
		&mut self.effects
	}
}

// TODO: Think about the allocation here
pub fn reactive<F, M, B>(factory: F) -> impl Markup<B>
where
	F: Fn(&mut StatefulContext<B, ReactiveExt<B>>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
{
	ReactiveComponentFactory {
		factory: Rc::new(factory),
		_b: PhantomData,
		_m: PhantomData,
		_e: PhantomData,
	}
}

pub struct ReactiveComponent<F, M, B, E: 'static>
where
	F: Fn(&mut StatefulContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Extension<WithCycle<B, E>>,
	E: MaybeExtension<WithEffects<B, E>>,
{
	state: Cell<State>,
	inner: RefCell<ReactiveComponentInner<F, M, B, E>>,
}

impl<F, M, B, E: 'static> Drop for ReactiveComponent<F, M, B, E>
where
	F: Fn(&mut StatefulContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Extension<WithCycle<B, E>>,
	E: MaybeExtension<WithEffects<B, E>>,
{
	fn drop(&mut self) {
		tracing::info!("Reactive component destroyed");
		let context = &self.inner.borrow_mut().context;
		if let Some(effects @ WithEffects { .. }) = context.ext.try_get() {
			effects.cleanup_effects_internal(context)
		}
	}
}

impl<F, M, B, E> ReactiveComponent<F, M, B, E>
where
	F: Fn(&mut StatefulContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Extension<WithCycle<B, E>>,
	E: MaybeExtension<WithEffects<B, E>>,
	E: MaybeExtension<WithReactions>,
	E: MaybeExtension<WithArena>,
{
	pub fn update(&self) {
		// Passing the same context
		let mut component = self.inner.borrow_mut();
		let component = &mut *component;

		let is_valid = match self.state.get() {
			State::Valid => true,
			State::Invalid(Invalid::Definitely) => false,
			State::Invalid(Invalid::Maybe) => {
				if let Some(with_reactions @ WithReactions { .. }) =
					component.context.ext.try_get_mut()
				{
					with_reactions.dependencies.are_valid()
				} else {
					true
				}
			}
		};

		if is_valid {
			self.state.set(State::Valid);
			return;
		}

		if let Some(with_effects @ WithEffects { .. }) = component.context.ext.try_get_mut() {
			with_effects.reset_effects_alive();
		}

		let mut next_markup = (component.factory)(&mut component.context);

		self.state.set(State::Valid);

		if let Some(with_reactions @ WithReactions { .. }) = component.context.ext.try_get_mut() {
			with_reactions.dependencies.swap(
				with_reactions
					.evaluation
					.replace(Evaluation::new(with_reactions.derived.clone()))
					.unwrap()
					.take(),
				&with_reactions.derived,
			);
		}

		if M::dynamic() {
			next_markup.diff(&mut component.markup, &component.context.tree);
		}

		component.markup = next_markup;

		if let Some(with_arena @ WithArena { .. }) = component.context.ext.try_get_mut() {
			with_arena.arena_prev.reset();
			std::mem::swap(&mut with_arena.arena, &mut with_arena.arena_prev);
		}

		let this = component.context.ext.get().this.clone();
		queue(move || {
			if let Some(c) = this.upgrade() {
				let context = c.context();
				// TODO move check up
				if let Some(with_effects @ WithEffects { .. }) = context.ext.try_get() {
					with_effects.run_effects(&context);
				}
			}
		});
	}
}

impl<F, M, B, E> observe::Reactive for ReactiveComponent<F, M, B, E>
where
	F: Fn(&mut StatefulContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Extension<WithCycle<B, E>>,
	E: MaybeExtension<WithEffects<B, E>>,
	E: MaybeExtension<WithReactions>,
	E: MaybeExtension<WithArena>,
{
	fn update(&self) {
		ReactiveComponent::update(self)
	}
}

impl<F, M, B, E> Derived for ReactiveComponent<F, M, B, E>
where
	F: Fn(&mut StatefulContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Extension<WithCycle<B, E>>,
	E: MaybeExtension<WithEffects<B, E>>,
	E: MaybeExtension<WithReactions>,
	E: MaybeExtension<WithArena>,
{
	fn invalidate(self: Rc<Self>, invalid: observe::Invalid) {
		if matches!(self.state.get(), State::Valid) {
			self.state.set(State::Invalid(invalid));
			unsafe {
				if !observe::in_batch() {
					panic!("Reactive component was invalidated outside of a `batch` call");
				}

				// FIXME: Components may be added in a wrong order, resulting
				//        in unnecessary computations (for example, if parent changes
				//        the view completely, there is no need to recompute children.
				CHANGED
					.borrow_mut()
					.push(Rc::downgrade(&self) as Weak<dyn observe::Reactive>)
			}
		}
	}
}

impl<F, M, B, E> Markup<B> for ReactiveComponentFactory<F, M, B, E>
where
	F: Fn(&mut StatefulContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Extension<WithCycle<B, E>>,
	E: MaybeExtension<WithReactions>,
	E: MaybeExtension<WithArena>,
	E: MaybeExtension<WithEffects<B, E>>,
	E: DynInit + 'static,
{
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn dynamic() -> bool {
		true
	}

	fn render(&mut self, tree: &Tree<B>) {
		let component = Rc::new_cyclic(|this: &Weak<ReactiveComponent<F, M, B, E>>| {
			struct P<B, E> {
				derived: Weak<dyn Derived>,
				this: Weak<dyn HasContext<B, E>>,
			}

			impl<B, E> Provider for P<B, E> {
				fn provide<'a>(&'a self, demand: &mut std::any::Demand<'a>) {
					demand.provide_value(self.derived.clone());
				}
			}

			let mut context = StatefulContext {
				tree: tree.clone(),
				ext: E::dyn_init(&P {
					this: this.clone() as Weak<dyn HasContext<B, E>>,
					derived: this.clone() as Weak<dyn Derived>,
				}),
			};

			if let Some(with_reactions @ WithReactions { .. }) = context.ext.try_get_mut() {
				with_reactions
					.evaluation
					.replace(Evaluation::new(this.clone()));
			}

			let mut markup = (self.factory)(&mut context);

			if let Some(with_arena @ WithArena { .. }) = context.ext.try_get_mut() {
				std::mem::swap(&mut with_arena.arena, &mut with_arena.arena_prev);
			}

			if let Some(with_reactions @ WithReactions { .. }) = context.ext.try_get_mut() {
				let derived = with_reactions.derived.clone();
				with_reactions.dependencies.swap(
					with_reactions
						.evaluation
						.replace(Evaluation::new(this.clone()))
						.unwrap()
						.take(),
					&derived,
				);
			}

			markup.render(tree);

			let component = this.clone();
			queue(move || {
				if let Some(c) = component.upgrade() {
					let context = &RefCell::borrow(&c.inner).context;
					if let Some(with_arena @ WithEffects { .. }) = context.ext.try_get() {
						with_arena.run_effects(&context);
					}
				}
			});

			ReactiveComponent {
				state: Cell::new(State::Valid),
				inner: RefCell::new(ReactiveComponentInner {
					context,
					markup,
					factory: self.factory.clone(),
				}),
			}
		});

		tree.data_mut().set(component);
	}

	fn diff(&mut self, _prev: &mut Self, tree: &Tree<B>) {
		let component = tree.data().get::<Rc<ReactiveComponent<F, M, B, E>>>();

		{
			component.state.set(State::Invalid(Invalid::Definitely));
			let mut component = component.inner.borrow_mut();
			component.factory = self.factory.clone();
			component.context.tree = tree.clone();
		}

		component.update();
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		let component = tree
			.data_mut()
			.remove::<Rc<ReactiveComponent<F, M, B, E>>>();

		let mut inner = component.inner.borrow_mut();
		inner.markup.drop(tree, should_unmount);
		std::mem::drop(inner);

		// Clean itself
		if Self::has_own_node() {
			tree.clear()
		}
	}
}

impl<M, B, F, E> HasContext<B, E> for ReactiveComponent<F, M, B, E>
where
	M: Markup<B>,
	B: Backend,
	F: Fn(&mut StatefulContext<B, E>) -> M,
	E: Extension<WithCycle<B, E>>,
	E: MaybeExtension<WithEffects<B, E>>,
{
	fn context(&self) -> RefMut<'_, StatefulContext<B, E>> {
		RefMut::map(self.inner.borrow_mut(), |c| &mut c.context)
	}
}
