use std::any::TypeId;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use super::reactive::Extension;
use crate::web::reactive::{ReactiveContext, WithMemo};

pub struct Callback<T: ?Sized>(pub(crate) Rc<T>);

// impl<F: ?Sized, A: Tuple> Fn<A> for Callback<F>
// where
// 	F: Fn<A>,
// {
// 	extern "rust-call" fn call(&self, args: A) {
// 		self.0.call(args);
// 	}
// }

// impl<F: ?Sized, A: Tuple> FnMut<A> for Callback<F>
// where
// 	F: FnMut<A>,
// {
// 	extern "rust-call" fn call_mut(&mut self, _args: A) {
// 		panic!()
// 	}
// }

// impl<F: ?Sized, A: Tuple> FnOnce<A> for Callback<F>
// where
// 	F: FnOnce<A>,
// {
// 	type Output = ();
// 	extern "rust-call" fn call_once(self, _args: A) {
// 		panic!()
// 	}
// }

impl<T: ?Sized> PartialEq for Callback<T> {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

impl<T: ?Sized> Clone for Callback<T> {
	fn clone(&self) -> Self {
		Callback(self.0.clone())
	}
}

impl<T> Callback<T> {
	fn new(func: T) -> Self {
		Callback(Rc::new(func))
	}
}

impl<T: ?Sized> Deref for Callback<T> {
	type Target = Rc<T>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: ?Sized> Eq for Callback<T> {}

struct Callback0<F, M, E>
where
	F: Fn(&mut ReactiveContext<E>),
{
	func: F,
	memo: M,
	context: Weak<dyn crate::web::reactive::Renderable<E>>,
}

impl<F, M, E> Fn<()> for Callback0<F, M, E>
where
	F: Fn(&mut ReactiveContext<E>),
{
	extern "rust-call" fn call(&self, _args: ()) {
		if let Some(context) = self.context.upgrade() {
			(self.func)(&mut *context.context())
		}
	}
}

impl<F, M, E> FnMut<()> for Callback0<F, M, E>
where
	F: Fn(&mut ReactiveContext<E>),
{
	extern "rust-call" fn call_mut(&mut self, args: ()) {
		self.call(args)
	}
}

impl<F, M, E> FnOnce<()> for Callback0<F, M, E>
where
	F: Fn(&mut ReactiveContext<E>),
{
	extern "rust-call" fn call_once(self, args: ()) {
		self.call(args)
	}

	type Output = ();
}

struct Callback1<F, T, M, E>
where
	F: Fn(&mut ReactiveContext<E>, T),
{
	func: F,
	memo: M,
	context: Weak<dyn crate::web::reactive::Renderable<E>>,
	_t: PhantomData<T>,
}

impl<F, T, M, E> Fn<(T,)> for Callback1<F, T, M, E>
where
	F: Fn(&mut ReactiveContext<E>, T),
{
	extern "rust-call" fn call(&self, args: (T,)) {
		if let Some(context) = self.context.upgrade() {
			(self.func)(&mut *context.context(), args.0)
		}
	}
}

impl<F, T, M, E> FnMut<(T,)> for Callback1<F, T, M, E>
where
	F: Fn(&mut ReactiveContext<E>, T),
{
	extern "rust-call" fn call_mut(&mut self, args: (T,)) {
		self.call(args)
	}
}

impl<F, T, M, E> FnOnce<(T,)> for Callback1<F, T, M, E>
where
	F: Fn(&mut ReactiveContext<E>, T),
{
	extern "rust-call" fn call_once(self, args: (T,)) {
		self.call(args)
	}

	type Output = ();
}

impl<E: 'static> ReactiveContext<E>
where
	Self: Extension<WithMemo>,
{
	pub fn callback_0_eq<F, M>(&self, memo: M, func: F) -> Callback<dyn Fn()>
	where
		M: PartialEq + 'static,
		F: Fn(&mut Self) + 'static,
	{
		let type_id = TypeId::of::<F>();

		let with_memo: &WithMemo = self.try_extension().unwrap();
		if let Some(cb) = with_memo.memo.borrow_mut().get(&type_id) {
			let callback = Rc::downcast::<Callback0<F, M, E>>(cb.clone())
				.map_err(|_| ())
				.unwrap();

			if callback.memo == memo {
				return Callback(callback);
			}
		}

		let callback = Rc::new(Callback0 {
			func,
			memo,
			context: self.renderable.clone(),
		});

		with_memo
			.memo
			.borrow_mut()
			.insert(type_id, callback.clone());

		return Callback(callback);
	}

	pub fn callback_0<F>(&self, func: F) -> Callback<dyn Fn()>
	where
		F: Fn(&mut Self) + 'static,
	{
		self.callback_0_eq((), func)
	}

	pub fn callback_0_hash<F, M>(&self, memo: &M, func: F) -> Callback<dyn Fn()>
	where
		M: Hash,
		F: Fn(&mut Self) + 'static,
	{
		self.callback_0_eq(fxhash::hash64(memo), func)
	}

	pub fn callback_1_eq<F, M, T: 'static>(&self, memo: M, func: F) -> Callback<dyn Fn(T)>
	where
		M: PartialEq + 'static,
		F: Fn(&mut Self, T) + 'static,
	{
		let type_id = TypeId::of::<F>();

		let with_memo: &WithMemo = self.try_extension().unwrap();
		if let Some(cb) = with_memo.memo.borrow_mut().get(&type_id) {
			let callback = Rc::downcast::<Callback1<F, T, M, E>>(cb.clone())
				.map_err(|_| ())
				.unwrap();

			if callback.memo == memo {
				return Callback(callback);
			}
		}

		let callback = Rc::new(Callback1 {
			func,
			memo,
			context: self.renderable.clone(),
			_t: PhantomData,
		});

		with_memo
			.memo
			.borrow_mut()
			.insert(type_id, callback.clone());

		return Callback(callback);
	}

	pub fn callback_1<F, T: 'static>(&self, func: F) -> Callback<dyn Fn(T)>
	where
		F: Fn(&mut Self, T) + 'static,
	{
		self.callback_1_eq((), func)
	}

	pub fn callback_1_hash<F, M, T: 'static>(&self, memo: &M, func: F) -> Callback<dyn Fn(T)>
	where
		M: Hash,
		F: Fn(&mut Self, T) + 'static,
	{
		self.callback_1_eq(fxhash::hash64(memo), func)
	}
}
