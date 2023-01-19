use std::ops::Range;

use bump::bump_in;
pub use callback::Callback;
use event::{on, on_if};
use observe::{batch, Var};
use reactive::reactive;

use self::helpers::cursor::Cursor;
use crate::{console_log, Backend, Markup};

pub mod api;
pub mod attr;
pub mod bump;
pub mod callback;
pub mod constant;
pub mod dispatch;
pub mod effect;
pub mod event;
pub mod helpers;
pub mod portal;
pub mod reactive;
pub mod root;
pub mod routing;
pub mod tag;
pub mod text;

pub mod prelude {
	pub use crate::reference::reference;
	pub use crate::web::attr::{attr, classlist, classname};
	pub use crate::web::callback::Callback;
	pub use crate::web::event::{on, on_if};
	pub use crate::web::html::*;
	pub use crate::web::reactive::reactive;
	pub use crate::Markup;
}

pub use attr::*;
use html::*;
pub use tag::html;

pub type Target = WebSys;

#[derive(Debug)]
pub struct WebSys {}

impl WebSys {}

impl Backend for WebSys {
	type Node = web_sys::Node;
	type Cursor = Cursor;
	type Event = web_sys::Event;

	fn replace(node: &Self::Node, prev: &Self::Node) {
		if let Some(parent_element) = prev.parent_element() {
			parent_element.replace_child(node, prev).unwrap();
		}
	}

	fn insert(cursor: Cursor, node: &Self::Node) {
		cursor.range.insert_node(node).unwrap()
	}

	fn cursor_after(node: &Self::Node) -> Self::Cursor {
		Cursor::after(node).unwrap()
	}

	fn cursor_beginning_of(node: &Self::Node) -> Self::Cursor {
		Cursor::beginning_of(node).unwrap()
	}
}
// impl Backend for Json {}

#[derive(Hash, PartialEq)]
struct Counter(usize);

fn counter(first: usize) -> impl Markup {
	reactive(move |cx| {
		cx.with(Counter(first));

		bump_in(cx, |bump| {
			let str = bumpalo::format!(in bump, "string {}", 10).into_bump_str();

			let counter: &Counter = cx.get();
			let evenodd = if counter.0 % 2 == 0 { "even" } else { "odd" };

			cx.effect_hash_clean(&evenodd, move |cx| {
				web_sys::console::log_1(&format!("Run {}", evenodd).into());
				move |cx| {
					web_sys::console::log_1(&format!("Cleanup {}", evenodd).into());
				}
			});

			div((
				classname(evenodd),
				"Counter ",
				str,
				counter.0.to_string(),
				on("click", cx.wrap(|cx, _| cx.update::<Counter>(|c| c.0 += 1))),
			))
		})
	})
}

#[derive(Eq, PartialEq)]
struct ButtonProps {
	value: usize,
	on_click: Option<Callback<dyn Fn(usize)>>,
}

fn button(props: ButtonProps) -> impl Markup {
	div((
		"Button",
		props.value.to_string(),
		on_if(props.on_click, "click", move |f, _| f(props.value)),
	))
}

fn app1() -> impl Markup {
	reactive(|cx| {
		cx.with(0 as usize);

		let on_click = cx.callback_1(|cx, value| {
			web_sys::console::log_1(&format!("Callback {}", value).into());
			cx.set(value + 1);
		});

		button(ButtonProps {
			value: *cx.get(),
			on_click: Some(on_click),
		})
	})
}

fn wrapper(children: impl Markup) -> impl Markup {
	div(children)
}

fn app2() -> impl Markup {
	wrapper("String")
}

struct A(bool);
struct B(bool);
struct C(bool);
struct D(bool);
struct E(bool);

fn fragments() -> impl Markup {
	reactive(|cx| {
		cx.with(A(false));
		cx.with(B(false));
		cx.with(C(false));
		cx.with(D(false));

		let a = cx.get::<A>();
		let b = cx.get::<B>();
		let c = cx.get::<C>();
		let d = cx.get::<D>();

		(
			a.0.then(|| ("a", "a", "a", classname("a"))),
			b.0.then(|| ("b", "b", "b")),
			c.0.then(|| ("c", "c", "c")),
			d.0.then(|| ("d", "d", "d")),
			(
				input((
					attr("type", "checkbox"),
					on("change", cx.wrap(|cx, _| cx.set::<A>(A(!cx.get::<A>().0)))),
				)),
				input((
					attr("type", "checkbox"),
					on("change", cx.wrap(|cx, _| cx.set::<B>(B(!cx.get::<B>().0)))),
				)),
				input((
					attr("type", "checkbox"),
					on("change", cx.wrap(|cx, _| cx.set::<C>(C(!cx.get::<C>().0)))),
				)),
				input((
					attr("type", "checkbox"),
					on("change", cx.wrap(|cx, _| cx.set::<D>(D(!cx.get::<D>().0)))),
				)),
			),
		)
	})
}

pub fn app_list(range: Range<usize>) -> impl Markup {
	crate::list::list(range.map(|i| (i, i)), |i, _| (span(i.to_string()), br(())))
}

fn variables() -> impl Markup {
	reactive(|cx| {
		cx.with(Var::new(true));

		let var = cx.get::<Var<bool>>();
		let text = if *var.get(cx) == true {
			"True"
		} else {
			"False"
		};

		cx.effect_hash_clean(text, |cx| {
			console_log!("Effect");
			|cx| {
				console_log!("Effect cleanup");
			}
		});

		(
			text,
			on(
				"click",
				cx.wrap(|cx, _e| {
					batch(|| cx.get::<Var<bool>>().update(|v| *v = !*v));
				}),
			),
		)
	})
}

// #[cfg_attr(not(test), wasm_bindgen::prelude::wasm_bindgen(start))]
// pub fn main() {
// 	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
// 	tracing_wasm::set_as_global_default();

// 	let body = web_sys::window()
// 		.unwrap()
// 		.document()
// 		.unwrap()
// 		.body()
// 		.unwrap();

// 	let document = web_sys::window().unwrap().document().unwrap();
// 	let div: HtmlElement = document.create_element("div").unwrap().unchecked_into();

// 	body.append_child(&div).unwrap();

// 	// let mut root = Root::render(fragment(), div);
// 	let mut root = Root::render(variables(), div);

// 	// root.update(app_list(20..30));

// 	// web::console_log!("Tree {:#?}", &root.tree);

// 	std::mem::forget(root);
// }
