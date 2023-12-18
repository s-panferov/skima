use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::error::Request;
use std::marker::PhantomData;
use std::rc::Weak;

use bumpalo::Bump;
use indexmap::IndexMap;

use super::effect::EffectContext;
use super::reactive::queue;
use super::WebSys;
use crate::action::Action;
use crate::anydata::{AnyData, Envelope};
use crate::tree::Tree;
use crate::Backend;

pub struct MemoRef<T: Envelope> {
	_t: PhantomData<&'static T>,
	key: u64,
}

impl<T: Envelope> MemoRef<T> {
	pub fn deref<B: Backend, E: Extension<WithMemo>>(
		&self,
		context: &StatefulContext<B, E>,
	) -> T::Output {
		let memo = context.ext.get();
		memo.memo.borrow().get_with_key::<T>(self.key)
	}
}

impl<T: Envelope> Clone for MemoRef<T> {
	fn clone(&self) -> Self {
		MemoRef {
			_t: PhantomData,
			key: self.key,
		}
	}
}

impl<T: Envelope> Copy for MemoRef<T> {}

#[derive(Default)]
pub struct WithArena {
	pub(crate) arena: Bump,
	pub(crate) arena_prev: Bump,
}

#[derive(Default)]
pub struct WithMemo {
	pub(crate) memo: RefCell<AnyData>,
}

#[derive(Default)]
pub struct WithState {
	pub(crate) state: HashMap<TypeId, Box<dyn Any>>,
}

pub struct WithEffects<B, E> {
	pub(crate) effects: RefCell<IndexMap<TypeId, EffectContext<B, E>>>,
}

impl<B, E> Default for WithEffects<B, E> {
	fn default() -> Self {
		Self {
			effects: Default::default(),
		}
	}
}

pub struct WithCycle<B, E> {
	pub this: Weak<dyn HasContext<B, E>>,
}

pub trait HasContext<B: Backend, E> {
	fn context(&self) -> RefMut<StatefulContext<B, E>>;
}

pub struct StatefulContext<B: Backend = WebSys, E = DefaultExt<B>> {
	pub(crate) tree: Tree<B>,
	pub(crate) ext: E,
}

pub struct DefaultExt<B> {
	pub effects: WithEffects<B, Self>,
	pub memo: WithMemo,
	pub state: WithState,
}

impl<B> Default for DefaultExt<B> {
	fn default() -> Self {
		Self {
			effects: Default::default(),
			memo: Default::default(),
			state: Default::default(),
		}
	}
}

pub trait DynInit {
	fn dyn_init(req: &dyn std::error::Error) -> Self;
}

impl<B> DynInit for DefaultExt<B> {
	fn dyn_init(req: &dyn std::error::Error) -> Self {
		DefaultExt {
			effects: Default::default(),
			memo: Default::default(),
			state: Default::default(),
		}
	}
}

impl<B> Extension<WithState> for DefaultExt<B> {
	fn get(&self) -> &WithState {
		&self.state
	}

	fn get_mut(&mut self) -> &mut WithState {
		&mut self.state
	}
}

impl<B> MaybeExtension<WithEffects<B, Self>> for DefaultExt<B> {
	fn try_get(&self) -> Option<&WithEffects<B, Self>> {
		Some(&self.effects)
	}

	fn try_get_mut(&mut self) -> Option<&mut WithEffects<B, Self>> {
		Some(&mut self.effects)
	}
}

impl<B> MaybeExtension<WithArena> for DefaultExt<B> {
	fn try_get(&self) -> Option<&WithArena> {
		None
	}

	fn try_get_mut(&mut self) -> Option<&mut WithArena> {
		None
	}
}

impl<B> Extension<WithEffects<B, DefaultExt<B>>> for DefaultExt<B> {
	fn get(&self) -> &WithEffects<B, DefaultExt<B>> {
		&self.effects
	}
	fn get_mut(&mut self) -> &mut WithEffects<B, DefaultExt<B>> {
		&mut self.effects
	}
}

impl<B> Extension<WithMemo> for DefaultExt<B> {
	fn get(&self) -> &WithMemo {
		&self.memo
	}
	fn get_mut(&mut self) -> &mut WithMemo {
		&mut self.memo
	}
}

pub trait Extension<T> {
	fn get(&self) -> &T;
	fn get_mut(&mut self) -> &mut T;
}

pub trait MaybeExtension<T> {
	fn try_get(&self) -> Option<&T>;
	fn try_get_mut(&mut self) -> Option<&mut T>;
	fn has(&self) -> bool {
		self.try_get().is_some()
	}
}

impl<B: Backend + 'static, E: 'static> StatefulContext<B, E> {
	pub fn tree(&self) -> &Tree<B> {
		&self.tree
	}

	pub fn dispatch<T: Action>(&self, action: T) {
		let action = Box::new(action) as Box<dyn Action>;
		let tree = self.tree.clone();
		// FIXME: do we need a queue?
		queue(move || tree.dispatch(action))
	}

	// FIXME: Monomorphization
	pub fn env<T: Envelope>(&self) -> T::Output {
		self.try_env::<T>().unwrap_or_else(|| {
			panic!("No data of type {} the context", std::any::type_name::<T>());
		})
	}

	pub fn try_env<T: Envelope>(&self) -> Option<T::Output> {
		let mut cursor: Option<Tree<B>> = Some(self.tree.clone());

		while let Some(tree) = cursor {
			if let Some(data) = tree.data().try_get::<T>() {
				return Some(data);
			}
			cursor = tree.parent.clone();
		}

		None
	}

	pub fn memo<T: Envelope, F: FnOnce() -> T + 'static>(&mut self, func: F) -> T::Output
	where
		E: Extension<WithMemo>,
	{
		let with_memo: &WithMemo = self.ext.get();
		let mut memo = with_memo.memo.borrow_mut();
		let key = fxhash::hash64(&(TypeId::of::<T>(), TypeId::of::<F>()));
		if let Some(item) = memo.try_with_key::<T>(key) {
			item
		} else {
			let t = func();
			memo.set_with_key::<T>(key, t);
			memo.get_with_key::<T>(key)
		}
	}

	pub fn memo_ref<T: Envelope, F: FnOnce() -> T + 'static>(&mut self, func: F) -> MemoRef<T>
	where
		E: Extension<WithMemo>,
	{
		let with_memo: &WithMemo = self.ext.get();
		let mut memo = with_memo.memo.borrow_mut();
		let key = fxhash::hash64(&(TypeId::of::<T>(), TypeId::of::<F>()));

		if memo.try_with_key::<T>(key).is_none() {
			let t = func();
			memo.set_with_key::<T>(key, t);
		}

		MemoRef {
			key,
			_t: PhantomData,
		}
	}

	pub fn wrap_0<F>(&self, func: F) -> impl Fn()
	where
		F: Fn(&mut Self),
		E: Extension<WithCycle<B, E>>,
	{
		let this = self.ext.get().this.clone();
		move || {
			if let Some(this) = this.upgrade() {
				(func)(&mut this.context())
			}
		}
	}

	pub fn wrap_res_0<F, R>(&self, func: F) -> impl Fn() -> R
	where
		F: Fn(&mut Self) -> R,
		E: Extension<WithCycle<B, E>>,
	{
		let this = self.ext.get().this.clone();
		move || {
			if let Some(this) = this.upgrade() {
				(func)(&mut this.context())
			} else {
				panic!("Context has gone, cannot return a result")
			}
		}
	}

	pub fn wrap_1<F, T>(&self, func: F) -> impl Fn(T)
	where
		F: Fn(&mut Self, T),
		E: Extension<WithCycle<B, E>>,
	{
		let this = self.ext.get().this.clone();
		move |v| {
			if let Some(this) = this.upgrade() {
				(func)(&mut this.context(), v)
			}
		}
	}
}

// trait TyEq {}

// impl<T> TyEq for (T, T) {}

// impl<BACKEND: Backend, A: 'static, T: 'static> ExtensionMut<T> for StatefulContext<BACKEND, (A,)>
// where
// 	(T, A): TyEq,
// {
// 	fn extension_mut(&mut self) -> &mut T {
// 		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
// 			return t;
// 		}

// 		dont_panic!()
// 	}

// 	fn try_extension_mut(&mut self) -> Option<&mut T> {
// 		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
// 			return Some(t);
// 		}

// 		None
// 	}
// }

// impl<BACKEND: Backend, A: 'static, B: 'static, T: 'static> ExtensionMut<T>
// 	for StatefulContext<BACKEND, (A, B)>
// where
// 	(T, A): TyEq,
// {
// 	fn extension_mut(&mut self) -> &mut T {
// 		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
// 			return t;
// 		}

// 		dont_panic!()
// 	}

// 	fn try_extension_mut(&mut self) -> Option<&mut T> {
// 		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
// 			return Some(t);
// 		}

// 		None
// 	}
// }

// impl<BACKEND: Backend, A: 'static, B: 'static, C: 'static, T: 'static> ExtensionMut<T>
// 	for StatefulContext<BACKEND, (A, B, C)>
// {
// 	fn extension_mut(&mut self) -> &mut T {
// 		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&mut self.ext.2, &mut T) {
// 			return t;
// 		}

// 		dont_panic!()
// 	}

// 	fn try_extension_mut(&mut self) -> Option<&mut T> {
// 		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&mut self.ext.2, &mut T) {
// 			return Some(t);
// 		}

// 		None
// 	}
// }

// impl<BACKEND: Backend, A: 'static, B: 'static, C: 'static, D: 'static, T: 'static> ExtensionMut<T>
// 	for StatefulContext<BACKEND, (A, B, C, D)>
// {
// 	fn extension_mut(&mut self) -> &mut T {
// 		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&mut self.ext.2, &mut T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&mut self.ext.3, &mut T) {
// 			return t;
// 		}

// 		dont_panic!()
// 	}

// 	fn try_extension_mut(&mut self) -> Option<&mut T> {
// 		if let Ok(t) = cast!(&mut self.ext.0, &mut T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&mut self.ext.1, &mut T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&mut self.ext.2, &mut T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&mut self.ext.3, &mut T) {
// 			return Some(t);
// 		}

// 		None
// 	}
// }

// impl<BACKEND: Backend, A: 'static, T: 'static> Extension<T> for StatefulContext<BACKEND, (A,)> {
// 	fn extension(&self) -> &T {
// 		if let Ok(t) = cast!(&self.ext.0, &T) {
// 			return t;
// 		}

// 		dont_panic!()
// 	}

// 	fn try_extension(&self) -> Option<&T> {
// 		if let Ok(t) = cast!(&self.ext.0, &T) {
// 			return Some(t);
// 		}

// 		None
// 	}
// }

// impl<BACKEND: Backend, A: 'static, B: 'static, T: 'static> Extension<T>
// 	for StatefulContext<BACKEND, (A, B)>
// {
// 	fn extension(&self) -> &T {
// 		if let Ok(t) = cast!(&self.ext.0, &T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&self.ext.1, &T) {
// 			return t;
// 		}

// 		dont_panic!()
// 	}

// 	fn try_extension(&self) -> Option<&T> {
// 		if let Ok(t) = cast!(&self.ext.0, &T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&self.ext.1, &T) {
// 			return Some(t);
// 		}

// 		None
// 	}
// }

// impl<BACKEND: Backend, A: 'static, B: 'static, C: 'static, T: 'static> Extension<T>
// 	for StatefulContext<BACKEND, (A, B, C)>
// {
// 	fn extension(&self) -> &T {
// 		if let Ok(t) = cast!(&self.ext.0, &T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&self.ext.1, &T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&self.ext.2, &T) {
// 			return t;
// 		}

// 		dont_panic!()
// 	}

// 	fn try_extension(&self) -> Option<&T> {
// 		if let Ok(t) = cast!(&self.ext.0, &T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&self.ext.1, &T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&self.ext.2, &T) {
// 			return Some(t);
// 		}

// 		None
// 	}
// }

// impl<BACKEND: Backend, A: 'static, B: 'static, C: 'static, D: 'static, T: 'static> Extension<T>
// 	for StatefulContext<BACKEND, (A, B, C, D)>
// {
// 	fn extension(&self) -> &T {
// 		if let Ok(t) = cast!(&self.ext.0, &T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&self.ext.1, &T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&self.ext.2, &T) {
// 			return t;
// 		} else if let Ok(t) = cast!(&self.ext.3, &T) {
// 			return t;
// 		}

// 		dont_panic!()
// 	}

// 	fn try_extension(&self) -> Option<&T> {
// 		if let Ok(t) = cast!(&self.ext.0, &T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&self.ext.1, &T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&self.ext.2, &T) {
// 			return Some(t);
// 		} else if let Ok(t) = cast!(&self.ext.3, &T) {
// 			return Some(t);
// 		}

// 		None
// 	}
// }
