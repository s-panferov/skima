use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::tree::Tree;
use crate::web::{Callback, Markup, WebSys};

pub(crate) struct EventListener<F>
where
	F: Fn(web_sys::Event) + ?Sized,
{
	pub(crate) event: &'static str,
	pub(crate) callback: Rc<F>,
}

impl<F> EventListener<F>
where
	F: Fn(web_sys::Event),
{
	pub fn new(event: &'static str, func: F) -> Self {
		EventListener {
			event,
			callback: Rc::new(func),
		}
	}
}

struct EventListenerData<F>
where
	F: Fn(web_sys::Event) + ?Sized + 'static,
{
	func: RefCell<Rc<F>>,
	closure: Closure<dyn Fn(web_sys::Event)>,
}

impl<F> Markup<WebSys> for EventListener<F>
where
	F: Fn(web_sys::Event) + ?Sized + 'static,
{
	fn has_own_node() -> bool {
		false
	}

	fn dynamic() -> bool {
		true
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tracing::info!("Rendering event {}", self.event);
		tracing::info!("Event tree {:?}", tree);

		let data =
			Rc::<EventListenerData<F>>::new_cyclic(|this| {
				let data = this.clone();
				let closure = Closure::wrap(Box::new(move |event| {
					(data.upgrade().unwrap().func.borrow())(event)
				}) as Box<dyn Fn(web_sys::Event)>);

				EventListenerData {
					func: RefCell::new(self.callback.clone()),
					closure,
				}
			});

		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.add_event_listener_with_callback(&self.event, data.closure.as_ref().unchecked_ref())
			.unwrap();

		tree.set_data(data);
	}

	fn diff(&self, _prev: &Self, tree: &Tree<WebSys>) {
		let data = tree.data::<EventListenerData<F>>();
		*data.func.borrow_mut() = self.callback.clone();
	}

	fn drop(&self, tree: &Tree<WebSys>, _should_unmount: bool) {
		tracing::info!("Drop event {}", self.event);
		let data = tree.remove_data::<EventListenerData<F>>();
		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.remove_event_listener_with_callback(self.event, data.closure.as_ref().unchecked_ref())
			.unwrap()
	}
}

pub trait IntoEventCallback {
	type Fn: Fn(web_sys::Event) + ?Sized + 'static;
	fn into_fn(self) -> Rc<Self::Fn>;
}

impl<F> IntoEventCallback for F
where
	F: Fn(web_sys::Event) + 'static,
{
	type Fn = Self;
	fn into_fn(self) -> Rc<Self::Fn> {
		Rc::new(self)
	}
}

impl IntoEventCallback for Callback<dyn Fn(web_sys::Event)> {
	type Fn = dyn Fn(web_sys::Event);
	fn into_fn(self) -> Rc<Self::Fn> {
		self.0.clone()
	}
}

pub fn on(event: &'static str, callback: impl IntoEventCallback) -> impl Markup {
	EventListener {
		event,
		callback: callback.into_fn(),
	}
}

pub fn on_if<T: 'static, F>(val: Option<T>, event: &'static str, callback: F) -> impl Markup
where
	F: Fn(&T, web_sys::Event) + 'static,
{
	val.map(|val| EventListener {
		event,
		callback: Rc::new(move |v| (callback)(&val, v)),
	})
}
