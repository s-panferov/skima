use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::tree::Tree;
use crate::web::{Callback, Markup, WebSys};

pub trait EventCallback: Clone + 'static {
	fn type_id(&self) -> TypeId;
	fn eq(&self, other: &Self) -> bool;
	fn call(&self, event: web_sys::Event);
}

impl<T> EventCallback for Rc<T>
where
	T: Fn(web_sys::Event) + 'static,
{
	fn call(&self, event: web_sys::Event) {
		self(event)
	}

	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(self, other)
	}

	fn type_id(&self) -> TypeId {
		TypeId::of::<T>()
	}
}

pub(crate) struct EventListener<C>
where
	C: EventCallback,
{
	pub(crate) event: &'static str,
	key: u64,
	pub(crate) callback: C,
}

impl<C> EventListener<C>
where
	C: EventCallback,
{
	pub fn new(event: &'static str, callback: C) -> Self {
		let key = fxhash::hash64(&callback.type_id());
		EventListener {
			event,
			callback,
			key,
		}
	}
}

struct EventListenerData<C>
where
	C: EventCallback,
{
	func: RefCell<C>,
	closure: Closure<dyn Fn(web_sys::Event)>,
}

impl<C> Markup<WebSys> for EventListener<C>
where
	C: EventCallback,
{
	fn has_own_node() -> bool {
		false
	}

	fn dynamic() -> bool {
		true
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tracing::debug!("Rendering event {}", self.event);
		tracing::debug!("Event tree {:?}", tree);

		let data = Rc::<EventListenerData<C>>::new_cyclic(|this| {
			let data = this.clone();
			let closure = Closure::wrap(Box::new(move |event| {
				(data.upgrade().unwrap().func.borrow()).call(event)
			}) as Box<dyn Fn(web_sys::Event)>);

			EventListenerData {
				func: RefCell::new(self.callback.clone()),
				closure,
			}
		});

		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.add_event_listener_with_callback(self.event, data.closure.as_ref().unchecked_ref())
			.unwrap();

		tree.data_mut().set_with_key(self.key, data);
	}

	fn diff(&self, _prev: &Self, tree: &Tree<WebSys>) {
		let data = tree
			.data_mut()
			.get_with_key::<Rc<EventListenerData<C>>>(self.key);
		if !self.callback.eq(&_prev.callback) {
			*data.func.borrow_mut() = self.callback.clone();
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, _should_unmount: bool) {
		tracing::info!("Drop event {}", self.event);
		let data = tree
			.data_mut()
			.remove_with_key::<Rc<EventListenerData<C>>>(self.key);

		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.remove_event_listener_with_callback(self.event, data.closure.as_ref().unchecked_ref())
			.unwrap()
	}
}

pub trait IntoEventCallback {
	type Callback: EventCallback;
	fn into_callback(self) -> Self::Callback;
}

impl<F> IntoEventCallback for F
where
	F: Fn(web_sys::Event) + 'static,
{
	type Callback = Rc<Self>;
	fn into_callback(self) -> Self::Callback {
		Rc::new(self)
	}
}

impl IntoEventCallback for Callback<dyn Fn(web_sys::Event)> {
	type Callback = Self;
	fn into_callback(self) -> Self::Callback {
		self
	}
}

impl IntoEventCallback for Callback<dyn Fn()> {
	type Callback = Self;
	fn into_callback(self) -> Self::Callback {
		self
	}
}

pub fn on(event: &'static str, callback: impl IntoEventCallback) -> impl Markup {
	EventListener::new(event, callback.into_callback())
}
