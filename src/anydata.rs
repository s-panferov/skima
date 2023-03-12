use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default)]
pub struct AnyData {
	data: HashMap<u64, Rc<dyn Any>>,
}

pub trait Envelope: Any {
	type Output;
	fn to_dyn(self) -> Rc<dyn Any>;
	fn from_dyn(rc: Rc<dyn Any>) -> Self::Output;
}

impl<T: Any> Envelope for Rc<T> {
	type Output = Self;

	fn from_dyn(rc: Rc<dyn Any>) -> Self {
		rc.downcast().unwrap()
	}

	fn to_dyn(self) -> Rc<dyn Any> {
		self
	}
}

impl<T: 'static> Envelope for observe::Var<T> {
	type Output = Self;
	fn to_dyn(self) -> Rc<dyn Any> {
		self.into()
	}

	fn from_dyn(rc: Rc<dyn Any>) -> Self::Output {
		Self::try_from(rc).unwrap()
	}
}

impl<T: std::hash::Hash + 'static> Envelope for observe::Computed<T> {
	type Output = Self;
	fn to_dyn(self) -> Rc<dyn Any> {
		self.into()
	}

	fn from_dyn(rc: Rc<dyn Any>) -> Self::Output {
		Self::try_from(rc).unwrap()
	}
}

impl AnyData {
	#[inline]
	pub fn try_dyn_with_key(&self, key: u64) -> Option<Rc<dyn Any>> {
		self.data.get(&key).map(|p| p.clone())
	}

	#[inline]
	pub fn try_dyn<T: Any>(&self) -> Option<Rc<dyn Any>> {
		self.try_dyn_with_key(fxhash::hash64(&TypeId::of::<T>()))
	}

	#[inline]
	pub fn try_dyn_with_type_id(&self, tid: TypeId) -> Option<Rc<dyn Any>> {
		self.try_dyn_with_key(fxhash::hash64(&tid))
	}

	#[inline]
	pub fn get_with_key<T: Envelope>(&self, key: u64) -> T::Output {
		let data: Rc<dyn Any> = (*self.data.get(&key).as_ref().expect(&type_name::<T>())).clone();
		T::from_dyn(data)
	}

	#[inline]
	pub fn try_with_key<T: Envelope>(&self, key: u64) -> Option<T::Output> {
		self.data
			.get(&key)
			.as_ref()
			.map(|d| T::from_dyn((*d).clone()))
	}

	#[inline]
	pub fn remove_with_key<T: Envelope>(&mut self, key: u64) -> T::Output {
		let data: Rc<dyn Any> = self.data.remove(&key).unwrap();
		T::from_dyn(data)
	}

	#[inline]
	pub fn set_with_key<T: Envelope>(&mut self, key: u64, value: T) {
		self.data.insert(key, T::to_dyn(value));
	}

	#[inline]
	pub fn set_dyn_with_key(&mut self, key: u64, value: Rc<dyn Any>) {
		self.data.insert(key, value);
	}

	#[inline]
	pub fn set_dyn_with_type_id(&mut self, tid: TypeId, value: Rc<dyn Any>) {
		self.set_dyn_with_key(fxhash::hash64(&tid), value);
	}

	#[inline]
	pub fn set_dyn<T: Any>(&mut self, value: Rc<dyn Any>) {
		self.data.insert(fxhash::hash64(&TypeId::of::<T>()), value);
	}

	#[inline]
	pub fn get<T: Envelope>(&self) -> T::Output {
		self.get_with_key::<T>(fxhash::hash64(&TypeId::of::<T>()))
	}

	#[inline]
	pub fn try_get<T: Envelope>(&self) -> Option<T::Output> {
		self.try_with_key::<T>(fxhash::hash64(&TypeId::of::<T>()))
	}

	#[inline]
	pub fn remove<T: Envelope>(&mut self) -> T::Output {
		self.remove_with_key::<T>(fxhash::hash64(&TypeId::of::<T>()))
	}

	#[inline]
	pub fn set<T: Envelope>(&mut self, value: T) {
		self.set_with_key::<T>(fxhash::hash64(&TypeId::of::<T>()), value)
	}
}

pub struct DynRc<T>(T);

impl<T: 'static> Envelope for DynRc<T>
where
	T: Into<Rc<dyn Any>>,
	T: TryFrom<Rc<dyn Any>>,
{
	type Output = T;

	fn from_dyn(rc: Rc<dyn Any>) -> Self::Output {
		T::try_from(rc)
			.map_err(|_| ())
			.expect(std::any::type_name::<T>())
	}

	fn to_dyn(self) -> Rc<dyn Any> {
		self.0.into()
	}
}
