use std::any::TypeId;
use std::marker::PhantomData;

use downcast_rs::{impl_downcast, Downcast};

use super::context::{Extension, StatefulContext, WithEffects};
use crate::Backend;

pub struct EffectWithCleanup<F, C, D, B, E>
where
	F: FnOnce(&StatefulContext<B, E>) -> C + 'static,
	C: FnOnce(&StatefulContext<B, E>) + 'static,
	D: PartialEq + 'static,
{
	func: Option<F>,
	cleanup: Option<C>,
	dep: D,
	_e: PhantomData<*const E>,
	_b: PhantomData<*const B>,
}

pub struct Effect<F, D, B, E>
where
	F: FnOnce(&StatefulContext<B, E>) + 'static,
	D: PartialEq + 'static,
{
	func: Option<F>,
	dep: D,
	_e: PhantomData<*const E>,
	_b: PhantomData<*const B>,
}

pub trait Effective<B: Backend, E>: Downcast {
	fn run(&mut self, cx: &StatefulContext<B, E>);
	fn cleanup(&mut self, cx: &StatefulContext<B, E>);
}

impl_downcast!(Effective<B, E> where B: Backend);

impl<F, D, B, E> Effective<B, E> for Effect<F, D, B, E>
where
	F: FnOnce(&StatefulContext<B, E>) + 'static,
	D: PartialEq + 'static,
	B: Backend + 'static,
	E: 'static,
{
	fn run(&mut self, cx: &StatefulContext<B, E>) {
		(self.func.take().unwrap())(cx)
	}

	fn cleanup(&mut self, _cx: &StatefulContext<B, E>) {}
}

impl<F, C, D, B, E> Effective<B, E> for EffectWithCleanup<F, C, D, B, E>
where
	F: FnOnce(&StatefulContext<B, E>) -> C + 'static,
	C: FnOnce(&StatefulContext<B, E>) + 'static,
	D: PartialEq + 'static,
	B: Backend + 'static,
	E: 'static,
{
	fn run(&mut self, cx: &StatefulContext<B, E>) {
		let cleanup = (self.func.take().unwrap())(cx);
		self.cleanup = Some(cleanup);
	}

	fn cleanup(&mut self, _cx: &StatefulContext<B, E>) {
		(self.cleanup.take().unwrap())(_cx)
	}
}

pub(crate) struct EffectContext<B, E> {
	current: Option<Box<dyn Effective<B, E>>>,
	scheduled: Option<Box<dyn Effective<B, E>>>,
	alive: bool,
}

impl<B, E> Default for EffectContext<B, E> {
	fn default() -> Self {
		Self {
			current: None,
			scheduled: None,
			alive: false,
		}
	}
}

impl<B: Backend, E> StatefulContext<B, E>
where
	E: Extension<WithEffects<B, E>>,
	B: 'static,
	E: 'static,
{
	pub fn effect<F>(&self, func: F)
	where
		F: FnOnce(&Self) + 'static,
	{
		let effect = Effect {
			func: Some(func),
			dep: 0,
			_b: PhantomData,
			_e: PhantomData,
		};

		let type_id = TypeId::of::<F>();
		let effects = self.ext.get();
		let mut effects = effects.effects.borrow_mut();
		let effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.scheduled = Some(Box::new(effect));
		effect_ctx.alive = true;
	}

	pub fn effect_clean<F, C>(&self, func: F)
	where
		F: Fn(&Self) -> C + 'static,
		C: Fn(&Self) + 'static,
	{
		let effect = EffectWithCleanup {
			func: Some(func),
			cleanup: None,
			dep: 0,
			_b: PhantomData,
			_e: PhantomData,
		};

		let type_id = TypeId::of::<F>();
		let effects = self.ext.get();
		let mut effects = effects.effects.borrow_mut();
		let effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.scheduled = Some(Box::new(effect));
		effect_ctx.alive = true;
	}

	pub fn effect_hash<F, D>(&self, deps: D, func: F)
	where
		F: Fn(&Self) + 'static,
		D: std::hash::Hash,
	{
		let hash = fxhash::hash64(&deps);
		self.effect_hash_internal(hash, func)
	}

	fn effect_hash_internal<F>(&self, hash: u64, func: F)
	where
		F: Fn(&Self) + 'static,
	{
		let effect = Effect {
			func: Some(func),
			dep: hash,
			_b: PhantomData,
			_e: PhantomData,
		};

		let type_id = TypeId::of::<F>();
		let effects = self.ext.get();
		let mut effects = effects.effects.borrow_mut();
		let effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.alive = true;

		if let Some(prev) = &effect_ctx.current {
			let prev = prev.downcast_ref::<Effect<F, u64, B, E>>().unwrap();
			if prev.dep != hash {
				effect_ctx.scheduled = Some(Box::new(effect));
			}
		} else {
			effect_ctx.scheduled = Some(Box::new(effect));
		}
	}

	pub fn effect_hash_clean<F, D, C>(&self, deps: D, func: F)
	where
		F: FnOnce(&Self) -> C + 'static,
		C: FnOnce(&Self) + 'static,
		D: std::hash::Hash,
	{
		let hash = fxhash::hash64(&deps);
		self.effect_hash_clean_internal(hash, func)
	}

	fn effect_hash_clean_internal<F, C>(&self, hash: u64, func: F)
	where
		F: FnOnce(&Self) -> C + 'static,
		C: FnOnce(&Self) + 'static,
	{
		let effect = EffectWithCleanup {
			func: Some(func),
			cleanup: None,
			dep: hash,
			_b: PhantomData,
			_e: PhantomData,
		};

		let type_id = TypeId::of::<F>();
		let effects = self.ext.get();
		let mut effects = effects.effects.borrow_mut();
		let effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.alive = true;
		if let Some(prev) = &effect_ctx.current {
			let prev = prev
				.downcast_ref::<EffectWithCleanup<F, C, u64, B, E>>()
				.unwrap();
			if prev.dep != hash {
				effect_ctx.scheduled = Some(Box::new(effect));
			}
		} else {
			effect_ctx.scheduled = Some(Box::new(effect));
		}
	}

	pub fn effect_once(&self, func: impl FnOnce(&Self) + 'static) {
		self.effect_eq(0, func)
	}

	pub fn effect_once_clean<F, C>(&self, func: F)
	where
		F: FnOnce(&Self) -> C + 'static,
		C: FnOnce(&Self) + 'static,
	{
		self.effect_eq_clean(0, func)
	}

	pub fn effect_eq<F, D: PartialEq + 'static>(&self, deps: D, func: F)
	where
		F: FnOnce(&Self) + 'static,
	{
		let type_id = TypeId::of::<F>();
		let effects = self.ext.get();
		let mut effects = effects.effects.borrow_mut();
		let effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.alive = true;
		if let Some(prev) = &effect_ctx.current {
			let prev = prev.downcast_ref::<Effect<F, D, B, E>>().unwrap();
			if prev.dep != deps {
				let effect = Effect {
					func: Some(func),
					dep: deps,
					_b: PhantomData,
					_e: PhantomData,
				};
				effect_ctx.scheduled = Some(Box::new(effect));
			}
		} else {
			let effect = Effect {
				func: Some(func),
				dep: deps,
				_b: PhantomData,
				_e: PhantomData,
			};
			effect_ctx.scheduled = Some(Box::new(effect));
		}
	}

	pub fn effect_eq_clean<F, D, C>(&self, deps: D, func: F)
	where
		F: FnOnce(&Self) -> C + 'static,
		C: FnOnce(&Self) + 'static,
		D: PartialEq + 'static,
	{
		let type_id = TypeId::of::<F>();
		let effects = self.ext.get();
		let mut effects = effects.effects.borrow_mut();
		let effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.alive = true;
		if let Some(prev) = &effect_ctx.current {
			let prev = prev
				.downcast_ref::<EffectWithCleanup<F, C, D, B, E>>()
				.unwrap();
			if prev.dep != deps {
				let effect = EffectWithCleanup {
					func: Some(func),
					cleanup: None,
					dep: deps,
					_b: PhantomData,
					_e: PhantomData,
				};
				effect_ctx.scheduled = Some(Box::new(effect));
			}
		} else {
			let effect = EffectWithCleanup {
				func: Some(func),
				cleanup: None,
				dep: deps,
				_b: PhantomData,
				_e: PhantomData,
			};
			effect_ctx.scheduled = Some(Box::new(effect));
		}
	}
}

impl<B: Backend + 'static, E: 'static> WithEffects<B, E> {
	pub(crate) fn reset_effects_alive(&self) {
		for effect in self.effects.borrow_mut().values_mut() {
			effect.alive = false;
		}
	}

	/// Run all scheduled effects
	pub(crate) fn run_effects(&self, context: &StatefulContext<B, E>) {
		for effect in self.effects.borrow_mut().values_mut() {
			if !effect.alive {
				assert!(effect.current.is_some());
				assert!(effect.scheduled.is_none());
				let mut current = effect.current.take().unwrap();
				current.cleanup(context);
				continue;
			}

			if let Some(mut scheduled) = effect.scheduled.take() {
				if let Some(mut previous) = effect.scheduled.take() {
					previous.cleanup(context)
				}
				scheduled.run(context);
				effect.current = Some(scheduled);
			}
		}
	}

	pub(crate) fn cleanup_effects_internal(&self, context: &StatefulContext<B, E>) {
		for effect in self.effects.borrow_mut().values_mut() {
			if let Some(mut previous) = effect.current.take() {
				previous.cleanup(context)
			}
		}

		self.effects.borrow_mut().clear();
	}
}
