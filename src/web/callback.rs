use std::any::TypeId;
use std::hash::Hash;
use std::marker::{PhantomData, Unsize};
use std::ops::Deref;
use std::rc::Rc;

use super::context::{Extension, StatefulContext, WithMemo};
use super::event::EventCallback;
use crate::Backend;

pub struct Callback<T: ?Sized>(pub Rc<T>, TypeId);

impl<T> EventCallback for Callback<T>
where
	T: Fn(web_sys::Event) + 'static,
{
	fn call(&self, event: web_sys::Event) {
		(self.0)(event)
	}

	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}

	fn type_id(&self) -> TypeId {
		self.1
	}
}

impl EventCallback for Callback<dyn Fn(web_sys::Event)> {
	fn call(&self, event: web_sys::Event) {
		(self.0)(event)
	}

	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}

	fn type_id(&self) -> TypeId {
		self.1
	}
}

impl EventCallback for Callback<dyn Fn()> {
	fn call(&self, _: web_sys::Event) {
		(self.0)()
	}

	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}

	fn type_id(&self) -> TypeId {
		self.1
	}
}

impl<T: ?Sized> PartialEq for Callback<T> {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

impl<T: ?Sized> Hash for Callback<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		Rc::as_ptr(&self.0).hash(state);
	}
}

impl<T: ?Sized> Clone for Callback<T> {
	fn clone(&self) -> Self {
		Callback(self.0.clone(), self.1)
	}
}

impl<T: 'static> Callback<T> {
	pub fn new(func: T) -> Self {
		Callback(Rc::new(func), TypeId::of::<T>())
	}

	pub fn wrap(func: Rc<T>) -> Self {
		Callback(func, TypeId::of::<T>())
	}

	pub fn to_dyn<U: ?Sized>(self) -> Callback<U>
	where
		T: Unsize<U>,
	{
		Callback(self.0 as Rc<U>, self.1)
	}
}

impl<T: ?Sized> Callback<T> {
	pub fn as_ptr(&self) -> *const T {
		Rc::as_ptr(&self.0)
	}
}

impl<T: ?Sized> Deref for Callback<T> {
	type Target = Rc<T>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: ?Sized> Eq for Callback<T> {}

struct Callback0<F, R, M>
where
	F: Fn() -> R,
{
	func: F,
	memo: M,
}

impl<F, R, M> Fn<()> for Callback0<F, R, M>
where
	F: Fn() -> R,
	R: Default,
{
	extern "rust-call" fn call(&self, _args: ()) -> R {
		(self.func)()
	}
}

impl<F, R, M> FnMut<()> for Callback0<F, R, M>
where
	F: Fn() -> R,
	R: Default,
{
	extern "rust-call" fn call_mut(&mut self, args: ()) -> R {
		self.call(args)
	}
}

impl<F, R, M> FnOnce<()> for Callback0<F, R, M>
where
	F: Fn() -> R,
	R: Default,
{
	extern "rust-call" fn call_once(self, args: ()) -> R {
		self.call(args)
	}

	type Output = R;
}

struct Callback1<F, R, T, M>
where
	F: Fn(T) -> R,
{
	func: F,
	memo: M,
	_t: PhantomData<T>,
}

impl<F, R, T, M> Fn<(T,)> for Callback1<F, R, T, M>
where
	F: Fn(T) -> R,
	R: Default,
{
	extern "rust-call" fn call(&self, args: (T,)) -> R {
		(self.func)(args.0)
	}
}

impl<F, R, T, M> FnMut<(T,)> for Callback1<F, R, T, M>
where
	F: Fn(T) -> R,
	R: Default,
{
	extern "rust-call" fn call_mut(&mut self, args: (T,)) -> R {
		self.call(args)
	}
}

impl<F, R, T, M> FnOnce<(T,)> for Callback1<F, R, T, M>
where
	F: Fn(T) -> R,
	R: Default,
{
	extern "rust-call" fn call_once(self, args: (T,)) -> R {
		self.call(args)
	}

	type Output = R;
}

impl<B: Backend + 'static, E: 'static> StatefulContext<B, E>
where
	E: Extension<WithMemo>,
{
	pub fn callback_0_eq<F, R, M>(&self, memo: M, func: F) -> Callback<dyn Fn() -> R>
	where
		M: PartialEq + 'static,
		R: Default + 'static,
		F: Fn() -> R + 'static,
	{
		let type_id = TypeId::of::<F>();
		let with_memo: &WithMemo = self.ext.get();
		if let Some(cb) = with_memo.memo.borrow_mut().try_dyn_with_type_id(type_id) {
			let callback = Rc::downcast::<Callback0<F, R, M>>(cb.clone())
				.map_err(|_| ())
				.unwrap();

			if callback.memo == memo {
				return Callback(callback, type_id);
			}
		}

		let callback = Rc::new(Callback0 { func, memo });

		with_memo
			.memo
			.borrow_mut()
			.set_dyn_with_type_id(type_id, callback.clone());

		Callback(callback, type_id)
	}

	pub fn callback_0<F>(&self, func: F) -> Callback<dyn Fn()>
	where
		F: Fn() + 'static,
	{
		self.callback_0_eq((), func)
	}

	pub fn callback_0_hash<F, M>(&self, memo: &M, func: F) -> Callback<dyn Fn()>
	where
		M: Hash,
		F: Fn() + 'static,
	{
		self.callback_0_eq(fxhash::hash64(memo), func)
	}

	pub fn callback_1_eq<F, R, M, T: 'static>(&self, memo: M, func: F) -> Callback<dyn Fn(T) -> R>
	where
		M: PartialEq + 'static,
		R: Default + 'static,
		F: Fn(T) -> R + 'static,
	{
		let type_id = TypeId::of::<F>();

		let with_memo: &WithMemo = self.ext.get();
		if let Some(cb) = with_memo.memo.borrow_mut().try_dyn_with_type_id(type_id) {
			let callback = Rc::downcast::<Callback1<F, R, T, M>>(cb.clone())
				.map_err(|_| ())
				.unwrap();

			if callback.memo == memo {
				return Callback(callback, type_id);
			}
		}

		let callback = Rc::new(Callback1 {
			func,
			memo,
			_t: PhantomData,
		});

		with_memo
			.memo
			.borrow_mut()
			.set_dyn_with_type_id(type_id, callback.clone());

		Callback(callback, type_id)
	}

	pub fn callback_1<F, R, T: 'static>(&self, func: F) -> Callback<dyn Fn(T) -> R>
	where
		F: Fn(T) -> R + 'static,
		R: Default + 'static,
	{
		self.callback_1_eq((), func)
	}

	pub fn callback_1_hash<F, M, R, T: 'static>(
		&self,
		memo: &M,
		func: F,
	) -> Callback<dyn Fn(T) -> R>
	where
		M: Hash,
		R: Default + 'static,
		F: Fn(T) -> R + 'static,
	{
		self.callback_1_eq(fxhash::hash64(memo), func)
	}
}
