use crate::{Backend, Markup};

pub struct EffectWithCleanup<F, C, D>
where
	F: FnOnce() -> C + 'static,
	C: FnOnce() + 'static,
	D: PartialEq + 'static,
{
	func: Option<F>,
	cleanup: Option<C>,
	dep: D,
}

pub struct EveryTime;

impl PartialEq for EveryTime {
	fn eq(&self, _: &Self) -> bool {
		false
	}
}

pub fn effect<F>(func: F) -> Effect<F, EveryTime>
where
	F: FnOnce() + 'static,
{
	Effect {
		dep: EveryTime,
		func: Some(func),
	}
}

pub fn effect_once<F>(func: F) -> Effect<F, u64>
where
	F: FnOnce() + 'static,
{
	Effect {
		dep: 0,
		func: Some(func),
	}
}

pub fn effect_eq<F, D>(dep: D, func: F) -> Effect<F, D>
where
	F: FnOnce() + 'static,
	D: std::cmp::Eq,
{
	Effect {
		dep,
		func: Some(func),
	}
}

pub fn effect_hash<F, D>(dep: &D, func: F) -> Effect<F, u64>
where
	F: FnOnce() + 'static,
	D: std::hash::Hash,
{
	let hash = fxhash::hash64(&dep);
	Effect {
		dep: hash,
		func: Some(func),
	}
}

pub fn effect_clean<F, C>(func: F) -> EffectWithCleanup<F, C, u64>
where
	F: FnOnce() -> C + 'static,
	C: FnOnce() + 'static,
{
	EffectWithCleanup {
		dep: 0,
		func: Some(func),
		cleanup: None,
	}
}

pub fn effect_hash_clean<F, C, D>(deps: &D, func: F) -> EffectWithCleanup<F, C, u64>
where
	F: FnOnce() -> C + 'static,
	C: FnOnce() + 'static,
	D: std::hash::Hash,
{
	let hash = fxhash::hash64(deps);
	EffectWithCleanup {
		dep: hash,
		func: Some(func),
		cleanup: None,
	}
}

pub fn effect_eq_clean<F, C, D>(dep: D, func: F) -> EffectWithCleanup<F, C, D>
where
	F: FnOnce() -> C + 'static,
	C: FnOnce() + 'static,
	D: std::cmp::PartialEq,
{
	EffectWithCleanup {
		dep,
		func: Some(func),
		cleanup: None,
	}
}

impl<F, C, D, B: Backend> Markup<B> for EffectWithCleanup<F, C, D>
where
	F: FnOnce() -> C + 'static,
	C: FnOnce() + 'static,
	D: PartialEq + 'static,
	B: Backend,
{
	fn render(&mut self, _: &crate::tree::Tree<B>) {
		self.cleanup = Some((self.func.take().unwrap())());
	}

	fn diff(&mut self, prev: &mut Self, _: &crate::tree::Tree<B>) {
		if prev.dep != self.dep {
			prev.cleanup.take().unwrap()();
			self.cleanup = Some((self.func.take().unwrap())());
		} else {
			self.cleanup = prev.cleanup.take();
		}
	}

	fn drop(&mut self, _: &crate::tree::Tree<B>, _: bool) {
		self.cleanup.take().unwrap()();
	}
}

pub struct Effect<F, D>
where
	F: FnOnce() + 'static,
	D: PartialEq + 'static,
{
	func: Option<F>,
	dep: D,
}

impl<F, D, B> Markup<B> for Effect<F, D>
where
	F: FnOnce() + 'static,
	D: PartialEq + 'static,
	B: Backend,
{
	fn render(&mut self, _: &crate::tree::Tree<B>) {
		self.func.take().unwrap()();
	}

	fn diff(&mut self, prev: &mut Self, _: &crate::tree::Tree<B>) {
		let func = self.func.take().unwrap();
		if prev.dep != self.dep {
			(func)()
		} else {
			// just drop the func
		}
	}

	fn drop(&mut self, _: &crate::tree::Tree<B>, _: bool) {}
}
