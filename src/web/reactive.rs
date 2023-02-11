use std::any::{type_name, Any, TypeId};
use std::cell::{Cell, RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use bumpalo::Bump;
use castaway::cast;
use indexmap::IndexMap;
use observe::{batch, Dependencies, Derived, Evaluation, Invalid, State, CHANGED};
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::JsValue;

use super::dispatch::ActionResult;
use super::WebSys;
use crate::action::Action;
use crate::anydata::Envelope;
use crate::dont_panic;
use crate::reference::Mutable;
use crate::tree::Tree;
use crate::web::effect::EffectContext;
use crate::web::{Backend, Markup};

#[derive(Default)]
pub struct WithArena {
	pub(crate) arena: Bump,
	pub(crate) arena_prev: Bump,
}

#[derive(Default)]
pub struct WithReactions {
	evaluation: Option<Evaluation>,
	dependencies: Dependencies,
}

#[derive(Default)]
pub struct WithMemo {
	pub(crate) memo: RefCell<HashMap<TypeId, Rc<dyn Any>>>,
}

#[derive(Default)]
pub struct WithState {
	state: HashMap<TypeId, Box<dyn Any>>,
}

pub struct ReactiveContext<B: Backend = WebSys, E = (WithMemo, WithReactions, WithState)> {
	pub(crate) effects: RefCell<IndexMap<TypeId, EffectContext<B, E>>>,
	pub(crate) renderable: Weak<dyn Renderable<B, E>>,
	derived: Weak<dyn Derived>,
	tree: Tree<B>,
	ext: E,
}

pub trait Extension<T> {
	fn extension(&self) -> &T;
	fn try_extension(&self) -> Option<&T>;
}

pub trait ExtensionMut<T> {
	fn extension_mut(&mut self) -> &mut T;
	fn try_extension_mut(&mut self) -> Option<&mut T>;
}

impl<BACKEND: Backend, A: 'static, T: 'static> ExtensionMut<T> for ReactiveContext<BACKEND, (A,)> {
	fn extension_mut(&mut self) -> &mut T {
		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
			return t;
		}

		dont_panic!()
	}

	fn try_extension_mut(&mut self) -> Option<&mut T> {
		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
			return Some(t);
		}

		return None;
	}
}

impl<BACKEND: Backend, A: 'static, B: 'static, T: 'static> ExtensionMut<T>
	for ReactiveContext<BACKEND, (A, B)>
{
	fn extension_mut(&mut self) -> &mut T {
		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
			return t;
		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
			return t;
		}

		dont_panic!()
	}

	fn try_extension_mut(&mut self) -> Option<&mut T> {
		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
			return Some(t);
		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
			return Some(t);
		}

		return None;
	}
}

impl<BACKEND: Backend, A: 'static, B: 'static, C: 'static, T: 'static> ExtensionMut<T>
	for ReactiveContext<BACKEND, (A, B, C)>
{
	fn extension_mut(&mut self) -> &mut T {
		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
			return t;
		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
			return t;
		} else if let Ok(t) = cast!(&mut self.ext.2, &mut T) {
			return t;
		}

		dont_panic!()
	}

	fn try_extension_mut(&mut self) -> Option<&mut T> {
		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
			return Some(t);
		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
			return Some(t);
		} else if let Ok(t) = cast!(&mut self.ext.2, &mut T) {
			return Some(t);
		}

		return None;
	}
}

impl<BACKEND: Backend, A: 'static, B: 'static, C: 'static, D: 'static, T: 'static> ExtensionMut<T>
	for ReactiveContext<BACKEND, (A, B, C, D)>
{
	fn extension_mut(&mut self) -> &mut T {
		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
			return t;
		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
			return t;
		} else if let Ok(t) = cast!(&mut self.ext.2, &mut T) {
			return t;
		} else if let Ok(t) = cast!(&mut self.ext.3, &mut T) {
			return t;
		}

		dont_panic!()
	}

	fn try_extension_mut(&mut self) -> Option<&mut T> {
		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
			return Some(t);
		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
			return Some(t);
		} else if let Ok(t) = cast!(&mut self.ext.2, &mut T) {
			return Some(t);
		} else if let Ok(t) = cast!(&mut self.ext.3, &mut T) {
			return Some(t);
		}

		return None;
	}
}

impl<BACKEND: Backend, A: 'static, T: 'static> Extension<T> for ReactiveContext<BACKEND, (A,)> {
	fn extension(&self) -> &T {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return t;
		}

		dont_panic!()
	}

	fn try_extension(&self) -> Option<&T> {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return Some(t);
		}

		return None;
	}
}

impl<BACKEND: Backend, A: 'static, B: 'static, T: 'static> Extension<T>
	for ReactiveContext<BACKEND, (A, B)>
{
	fn extension(&self) -> &T {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return t;
		} else if let Ok(t) = cast!(&self.ext.1, &T) {
			return t;
		}

		dont_panic!()
	}

	fn try_extension(&self) -> Option<&T> {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return Some(t);
		} else if let Ok(t) = cast!(&self.ext.1, &T) {
			return Some(t);
		}

		return None;
	}
}

impl<BACKEND: Backend, A: 'static, B: 'static, C: 'static, T: 'static> Extension<T>
	for ReactiveContext<BACKEND, (A, B, C)>
{
	fn extension(&self) -> &T {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return t;
		} else if let Ok(t) = cast!(&self.ext.1, &T) {
			return t;
		} else if let Ok(t) = cast!(&self.ext.2, &T) {
			return t;
		}

		dont_panic!()
	}

	fn try_extension(&self) -> Option<&T> {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return Some(t);
		} else if let Ok(t) = cast!(&self.ext.1, &T) {
			return Some(t);
		} else if let Ok(t) = cast!(&self.ext.2, &T) {
			return Some(t);
		}

		return None;
	}
}

impl<BACKEND: Backend, A: 'static, B: 'static, C: 'static, D: 'static, T: 'static> Extension<T>
	for ReactiveContext<BACKEND, (A, B, C, D)>
{
	fn extension(&self) -> &T {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return t;
		} else if let Ok(t) = cast!(&self.ext.1, &T) {
			return t;
		} else if let Ok(t) = cast!(&self.ext.2, &T) {
			return t;
		} else if let Ok(t) = cast!(&self.ext.3, &T) {
			return t;
		}

		dont_panic!()
	}

	fn try_extension(&self) -> Option<&T> {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return Some(t);
		} else if let Ok(t) = cast!(&self.ext.1, &T) {
			return Some(t);
		} else if let Ok(t) = cast!(&self.ext.2, &T) {
			return Some(t);
		} else if let Ok(t) = cast!(&self.ext.3, &T) {
			return Some(t);
		}

		return None;
	}
}

impl<BACKEND: Backend, T> AsRef<Evaluation> for ReactiveContext<BACKEND, T>
where
	Self: Extension<WithReactions>,
{
	fn as_ref(&self) -> &Evaluation {
		let with_reactions = self.extension();
		with_reactions.evaluation.as_ref().unwrap()
	}
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_name = queueMicrotask)]
	pub fn queue_microtask(closure: &JsValue);
}

pub fn queue<F: FnOnce() + 'static>(func: F) {
	queue_microtask(&Closure::once_into_js(move || func()));
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

impl<B: Backend, E> ReactiveContext<B, E>
where
	Self: Extension<WithMemo>,
{
	pub fn with_memo_rc<T: Any>(&mut self, func: impl FnOnce() -> Rc<T>) -> Rc<T> {
		let tid = TypeId::of::<T>();
		let with_memo: &WithMemo = self.try_extension().unwrap();
		let mut memo = with_memo.memo.borrow_mut();
		if let Some(item) = memo.get(&tid) {
			Rc::downcast(item.clone()).map_err(|_| ()).unwrap()
		} else {
			let t = func();
			memo.insert(tid, t.clone());
			t
		}
	}

	pub fn with_memo<T: Clone + Into<Rc<dyn Any>> + 'static>(
		&mut self,
		func: impl FnOnce() -> T,
	) -> T
	where
		Rc<dyn Any>: TryInto<T>,
	{
		let tid = TypeId::of::<T>();
		let with_memo: &WithMemo = self.try_extension().unwrap();
		let mut memo = with_memo.memo.borrow_mut();
		if let Some(item) = memo.get(&tid) {
			Rc::try_into(item.clone()).map_err(|_| ()).unwrap()
		} else {
			let t = func();
			memo.insert(tid, t.clone().into());
			t
		}
	}
}

impl<B: Backend, E> ReactiveContext<B, E>
where
	Self: ExtensionMut<WithState>,
	Self: Extension<WithState>,
{
	pub fn get<T: Any>(&self) -> &T {
		let state: &WithState = self.try_extension().unwrap();
		state
			.state
			.get(&TypeId::of::<T>())
			// FIXME: remove format
			.expect(&format!(
				"Item of type {} is now available",
				type_name::<T>()
			))
			.downcast_ref::<T>()
			.unwrap()
	}

	pub fn mutable<T: 'static>(&mut self, value: T) -> Mutable<T> {
		let state: &mut WithState = self.try_extension_mut().unwrap();
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

	pub fn with<T: Any>(&mut self, value: T) {
		let state: &mut WithState = self.try_extension_mut().unwrap();

		let tid = TypeId::of::<T>();
		if !state.state.contains_key(&tid) {
			state.state.insert(tid, Box::new(value) as Box<dyn Any>);
		}
	}

	pub fn with_fn<T: Any>(&mut self, func: impl FnOnce() -> T) {
		let state: &mut WithState = self.try_extension_mut().unwrap();

		let tid = TypeId::of::<T>();
		if !state.state.contains_key(&tid) {
			state.state.insert(tid, Box::new((func)()) as Box<dyn Any>);
		}
	}

	pub fn update<T: Any>(&mut self, func: impl FnOnce(&mut T)) {
		let state: &mut WithState = self.try_extension_mut().unwrap();
		state
			.state
			.entry(TypeId::of::<T>())
			.and_modify(|v| func(v.downcast_mut().unwrap()));

		self.derived
			.upgrade()
			.unwrap()
			.invalidate(Invalid::Definitely);

		// Hacky way to queue updates in a microtask
		queue(move || batch(|| {}))
	}

	pub fn set<T: Any>(&mut self, value: T) {
		let state: &mut WithState = self.try_extension_mut().unwrap();
		state
			.state
			.insert(TypeId::of::<T>(), Box::new(value) as Box<dyn Any>);

		self.derived
			.upgrade()
			.unwrap()
			.invalidate(Invalid::Definitely);

		// Hacky way to queue updates in a microtask
		queue(move || batch(|| {}))
	}
}

impl<B: Backend + 'static, E: 'static> ReactiveContext<B, E> {
	pub fn dispatch<T: Action>(&self, action: T) {
		let action = Box::new(action) as Box<dyn Action>;
		let tree = self.tree.clone();
		// FIXME: do we need a queue?
		queue(move || tree.dispatch(action))
	}

	// FIXME: Monomorphization
	pub fn env<T: Envelope>(&mut self) -> T::Output {
		let mut cursor: Option<Tree<B>> = Some(self.tree.clone());

		while let Some(tree) = cursor {
			if let Some(data) = tree.data().try_get::<T>() {
				return data;
			}
			cursor = tree.parent.clone();
		}

		panic!("No data of type {} the context", std::any::type_name::<T>());
	}

	pub fn wrap<F, T>(&self, func: F) -> impl Fn(T)
	where
		F: Fn(&mut Self, T),
	{
		let component = self.renderable.clone();
		move |v| {
			if let Some(component) = component.upgrade() {
				(func)(&mut component.context(), v)
			}
		}
	}
}

pub struct ReactiveComponentInner<
	F: Fn(&mut ReactiveContext<B, E>) -> M,
	M: Markup<B>,
	B: Backend,
	E,
> {
	context: ReactiveContext<B, E>,
	markup: M,
	factory: Rc<F>,
}

pub struct ReactiveComponentFactory<
	F: Fn(&mut ReactiveContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E = (),
> {
	factory: Rc<F>,
	_b: PhantomData<B>,
	_m: PhantomData<M>,
	_e: PhantomData<E>,
}

// TODO: Think about the allocation here
pub fn reactive<F, M, B>(factory: F) -> impl Markup<B>
where
	F: Fn(&mut ReactiveContext<B, (WithMemo, WithReactions, WithState)>) -> M + 'static,
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

pub(crate) trait Renderable<B: Backend, E> {
	fn update(&self);
	fn context(&self) -> RefMut<ReactiveContext<B, E>>;
}

pub struct ReactiveComponent<F, M, B, E: 'static>
where
	F: Fn(&mut ReactiveContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
{
	this: Weak<Self>,
	state: Cell<State>,
	inner: RefCell<ReactiveComponentInner<F, M, B, E>>,
}

impl<F, M, B, E: 'static> Drop for ReactiveComponent<F, M, B, E>
where
	F: Fn(&mut ReactiveContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
{
	fn drop(&mut self) {
		tracing::info!("Reactive component destroyed");
		self.inner.borrow().context.cleanup_effects_internal()
	}
}

impl<F, M, B, E> ReactiveComponent<F, M, B, E>
where
	F: Fn(&mut ReactiveContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Default + 'static,
	ReactiveContext<B, E>: ExtensionMut<WithReactions>,
	ReactiveContext<B, E>: ExtensionMut<WithArena>,
{
	pub fn enqueue_update(&self) {
		let component = self.this.clone();
		queue(move || {
			if let Some(c) = component.upgrade() {
				c.update()
			}
		});
	}

	pub fn update(&self) {
		// Passing the same context
		let mut component = self.inner.borrow_mut();
		let component = &mut *component;

		let is_valid = match self.state.get() {
			State::Valid => true,
			State::Invalid(Invalid::Definitely) => false,
			State::Invalid(Invalid::Maybe) => {
				if let Some(with_reactions @ WithReactions { .. }) =
					component.context.try_extension_mut()
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

		component.context.reset_effects_alive();

		let next_markup = (component.factory)(&mut component.context);

		self.state.set(State::Valid);

		let derived = component.context.derived.clone();
		if let Some(with_reactions @ WithReactions { .. }) = component.context.try_extension_mut() {
			with_reactions.dependencies.swap(
				with_reactions
					.evaluation
					.replace(Evaluation::new(derived.clone()))
					.unwrap()
					.take(),
				&derived,
			);
		}

		if M::dynamic() {
			next_markup.diff(&component.markup, &component.context.tree);
		}

		component.markup = next_markup;

		if let Some(with_arena @ WithArena { .. }) = component.context.try_extension_mut() {
			with_arena.arena_prev.reset();
			std::mem::swap(&mut with_arena.arena, &mut with_arena.arena_prev);
		}

		let component = component.context.renderable.clone();
		queue(move || {
			if let Some(c) = component.upgrade() {
				c.context().run_effects()
			}
		});
	}
}

impl<F, M, B, E> observe::Reactive for ReactiveComponent<F, M, B, E>
where
	F: Fn(&mut ReactiveContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Default + 'static,
	ReactiveContext<B, E>: ExtensionMut<WithReactions>,
	ReactiveContext<B, E>: ExtensionMut<WithArena>,
{
	fn update(&self) {
		self.enqueue_update()
	}
}

impl<F, M, B, E> Derived for ReactiveComponent<F, M, B, E>
where
	F: Fn(&mut ReactiveContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	E: Default + 'static,
	ReactiveContext<B, E>: ExtensionMut<WithReactions>,
	ReactiveContext<B, E>: ExtensionMut<WithArena>,
{
	fn invalidate(self: Rc<Self>, invalid: observe::Invalid) {
		if matches!(self.state.get(), State::Valid) {
			self.state.set(State::Invalid(invalid));
			unsafe {
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
	F: Fn(&mut ReactiveContext<B, E>) -> M + 'static,
	M: Markup<B> + 'static,
	B: Backend + 'static,
	ReactiveContext<B, E>: ExtensionMut<WithReactions>,
	ReactiveContext<B, E>: ExtensionMut<WithArena>,
	E: Default + 'static,
{
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn dynamic() -> bool {
		true
	}

	fn render(&self, tree: &Tree<B>) {
		let component = Rc::new_cyclic(|this: &Weak<ReactiveComponent<F, M, B, E>>| {
			let mut context = ReactiveContext {
				renderable: this.clone() as Weak<dyn Renderable<B, E>>,
				ext: E::default(),
				effects: Default::default(),
				derived: this.clone() as Weak<dyn Derived>,
				tree: tree.clone(),
			};

			if let Some(with_reactions @ WithReactions { .. }) = context.try_extension_mut() {
				with_reactions
					.evaluation
					.replace(Evaluation::new(this.clone()));
			}

			let markup = (self.factory)(&mut context);

			if let Some(with_arena @ WithArena { .. }) = context.try_extension_mut() {
				std::mem::swap(&mut with_arena.arena, &mut with_arena.arena_prev);
			}

			// FIXME: clone
			let derived = context.derived.clone();
			if let Some(with_reactions @ WithReactions { .. }) = context.try_extension_mut() {
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
					RefCell::borrow(&c.inner).context.run_effects();
				}
			});

			ReactiveComponent {
				this: this.clone(),
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

	fn diff(&self, _prev: &Self, tree: &Tree<B>) {
		let component = tree.data().get::<Rc<ReactiveComponent<F, M, B, E>>>();

		{
			component.state.set(State::Invalid(Invalid::Definitely));
			let mut component = component.inner.borrow_mut();
			component.factory = self.factory.clone();
			component.context.tree = tree.clone();
		}

		component.update();
	}

	fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
		let component = tree
			.data_mut()
			.remove::<Rc<ReactiveComponent<F, M, B, E>>>();

		let inner = component.inner.borrow_mut();
		inner.markup.drop(tree, should_unmount);
		std::mem::drop(inner);

		// Clean itself
		if Self::has_own_node() {
			tree.clear()
		}
	}
}

impl<M, B, F, E> Renderable<B, E> for ReactiveComponent<F, M, B, E>
where
	M: Markup<B>,
	B: Backend,
	F: Fn(&mut ReactiveContext<B, E>) -> M,
	E: Default + 'static,
	ReactiveContext<B, E>: ExtensionMut<WithReactions>,
	ReactiveContext<B, E>: ExtensionMut<WithArena>,
{
	fn update(&self) {
		ReactiveComponent::update(&self)
	}

	fn context(&self) -> RefMut<'_, ReactiveContext<B, E>> {
		RefMut::map(self.inner.borrow_mut(), |c| &mut c.context)
	}
}
