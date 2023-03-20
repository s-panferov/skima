use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use indexmap::IndexMap;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, Node};

pub fn input_event_value(ev: &Event) -> String {
	ev.target()
		.unwrap()
		.unchecked_into::<HtmlInputElement>()
		.value()
}

pub fn input_value(node: &Node) -> String {
	node.unchecked_ref::<HtmlInputElement>().value()
}

#[derive(Debug)]
pub struct ObservedMap<K, V>(IndexMap<K, V>);

impl<K, V> Default for ObservedMap<K, V> {
	fn default() -> Self {
		Self(IndexMap::default())
	}
}

impl<K, V> Deref for ObservedMap<K, V> {
	type Target = IndexMap<K, V>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<K, V> DerefMut for ObservedMap<K, V> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<K, V> Hash for ObservedMap<K, V>
where
	K: Hash,
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for key in self.keys() {
			key.hash(state)
		}
	}
}
