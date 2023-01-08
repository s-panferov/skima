use std::any::TypeId;
use std::marker::PhantomData;

use downcast_rs::{impl_downcast, Downcast};

use crate::web::reactive::ReactiveContext;

pub struct EffectWithCleanup<F, C, D, E>
where
	F: FnOnce(&ReactiveContext<E>) -> C + 'static,
	C: FnOnce(&ReactiveContext<E>) + 'static,
	D: PartialEq + 'static,
{
	func: Option<F>,
	cleanup: Option<C>,
	dep: D,
	_e: PhantomData<*const E>,
}

pub struct Effect<F, D, E>
where
	F: FnOnce(&ReactiveContext<E>) + 'static,
	D: PartialEq + 'static,
{
	func: Option<F>,
	dep: D,
	_e: PhantomData<*const E>,
}

pub trait Effective<E>: Downcast {
	fn run(&mut self, cx: &ReactiveContext<E>);
	fn cleanup(&mut self, cx: &ReactiveContext<E>);
}

impl_downcast!(Effective<E>);

impl<F, D, E> Effective<E> for Effect<F, D, E>
where
	F: FnOnce(&ReactiveContext<E>) + 'static,
	D: PartialEq + 'static,
	E: 'static,
{
	fn run(&mut self, cx: &ReactiveContext<E>) {
		(self.func.take().unwrap())(cx)
	}

	fn cleanup(&mut self, _cx: &ReactiveContext<E>) {}
}

impl<F, C, D, E> Effective<E> for EffectWithCleanup<F, C, D, E>
where
	F: FnOnce(&ReactiveContext<E>) -> C + 'static,
	C: FnOnce(&ReactiveContext<E>) + 'static,
	D: PartialEq + 'static,
	E: 'static,
{
	fn run(&mut self, cx: &ReactiveContext<E>) {
		let cleanup = (self.func.take().unwrap())(cx);
		self.cleanup = Some(cleanup);
	}

	fn cleanup(&mut self, _cx: &ReactiveContext<E>) {
		(self.cleanup.take().unwrap())(_cx)
	}
}

#[derive(Default)]
pub(crate) struct EffectContext<E> {
	current: Option<Box<dyn Effective<E>>>,
	scheduled: Option<Box<dyn Effective<E>>>,
	alive: bool,
}

impl<E: Default + 'static> ReactiveContext<E> {
	pub fn effect<F>(&self, func: F)
	where
		F: FnOnce(&Self) + 'static,
	{
		let effect = Effect {
			func: Some(func),
			dep: 0,
			_e: PhantomData,
		};

		let type_id = TypeId::of::<F>();
		let mut effects = self.effects.borrow_mut();
		let mut effect_ctx = effects.entry(type_id).or_insert(Default::default());

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
			_e: PhantomData,
		};

		let type_id = TypeId::of::<F>();
		let mut effects = self.effects.borrow_mut();
		let mut effect_ctx = effects.entry(type_id).or_insert(Default::default());

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
			_e: PhantomData,
		};

		let type_id = TypeId::of::<F>();
		let mut effects = self.effects.borrow_mut();
		let mut effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.alive = true;

		if let Some(prev) = &effect_ctx.current {
			let prev = prev.downcast_ref::<Effect<F, u64, E>>().unwrap();
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
			_e: PhantomData,
		};

		let type_id = TypeId::of::<F>();
		let mut effects = self.effects.borrow_mut();
		let mut effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.alive = true;
		if let Some(prev) = &effect_ctx.current {
			let prev = prev
				.downcast_ref::<EffectWithCleanup<F, C, u64, E>>()
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
		let mut effects = self.effects.borrow_mut();
		let mut effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.alive = true;
		if let Some(prev) = &effect_ctx.current {
			let prev = prev.downcast_ref::<Effect<F, D, E>>().unwrap();
			if prev.dep != deps {
				let effect = Effect {
					func: Some(func),
					dep: deps,
					_e: PhantomData,
				};
				effect_ctx.scheduled = Some(Box::new(effect));
			}
		} else {
			let effect = Effect {
				func: Some(func),
				dep: deps,
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
		let mut effects = self.effects.borrow_mut();
		let mut effect_ctx = effects.entry(type_id).or_insert(Default::default());

		effect_ctx.alive = true;
		if let Some(prev) = &effect_ctx.current {
			let prev = prev
				.downcast_ref::<EffectWithCleanup<F, C, D, E>>()
				.unwrap();
			if prev.dep != deps {
				let effect = EffectWithCleanup {
					func: Some(func),
					cleanup: None,
					dep: deps,
					_e: PhantomData,
				};
				effect_ctx.scheduled = Some(Box::new(effect));
			}
		} else {
			let effect = EffectWithCleanup {
				func: Some(func),
				cleanup: None,
				dep: deps,
				_e: PhantomData,
			};
			effect_ctx.scheduled = Some(Box::new(effect));
		}
	}
}

impl<E: 'static> ReactiveContext<E> {
	pub(crate) fn reset_effects_alive(&self) {
		let mut effects = self.effects.borrow_mut();
		for effect in effects.values_mut() {
			effect.alive = false;
		}
	}

	/// Run all scheduled effects
	pub(crate) fn run_effects(&self) {
		let mut effects = self.effects.borrow_mut();
		for effect in effects.values_mut() {
			if !effect.alive {
				assert!(effect.current.is_some());
				assert!(effect.scheduled.is_none());
				let mut current = effect.current.take().unwrap();
				current.cleanup(self);
				continue;
			}

			if let Some(mut scheduled) = effect.scheduled.take() {
				if let Some(mut previous) = effect.scheduled.take() {
					previous.cleanup(self)
				}
				scheduled.run(self);
				effect.current = Some(scheduled);
			}
		}
	}

	pub(crate) fn cleanup_effects_internal(&self) {
		let mut effects = self.effects.borrow_mut();
		for effect in effects.values_mut() {
			if let Some(mut previous) = effect.current.take() {
				previous.cleanup(self)
			}
		}

		effects.clear();
	}
}
