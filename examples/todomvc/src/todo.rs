use observe::macros::enclose;
use observe::Var;
use skima::combine::Either;
use skima::reference::Mutable;
use skima::web::context::StatefulContext;
use skima::web::prelude::*;
use skima::web::reactive::{queue, ReactiveExt};
use skima::web::WebSys;
use skima::Markup;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, KeyboardEvent, Node};

use crate::action::{TodoDelete, TodoEdit, TodoToggle};
use crate::helpers::input_value;

#[derive(Debug, Clone, Hash)]
pub struct TodoItem {
	pub id: Uuid,
	pub title: String,
	pub is_done: bool,
}

#[derive(Clone, Copy)]
struct Editing(bool);

pub fn todo_item(item: Var<TodoItem>) -> impl Markup {
	reactive(move |cx| {
		cx.with(Editing(false));

		let editing = cx.get::<Editing>().clone();
		let input_ref = cx.mutable::<Option<Node>>(None);

		let on_click = cx.callback_0({
			let input_ref = input_ref.clone();
			move |cx| {
				cx.update::<Editing>(|e| e.0 = !e.0);
				queue(enclose!((input_ref) move || {
					let _ = input_ref
					.unwrap()
					.unchecked_ref::<HtmlInputElement>()
					.focus();
				}))
			}
		});

		let on_keydown = cx.callback_1_hash(&item, {
			let item = item.clone();
			let input_ref = input_ref.clone();
			move |cx, ev: Event| {
				let ev = ev.unchecked_into::<KeyboardEvent>();
				match ev.key().as_ref() {
					"Enter" => {
						update_title(&item, &input_ref, cx);
						cx.update::<Editing>(|e| e.0 = !e.0);
					}
					_ => {}
				}
			}
		});

		let on_blur = cx.callback_0_hash(
			&item,
			enclose!((item, input_ref) move |cx| {
				update_title(&item, &input_ref, cx);
				cx.update::<Editing>(|e| e.0 = !e.0);
			}),
		);

		let on_destroy = cx.callback_0_hash(
			&item,
			enclose!((item) move |cx| {
				destroy(&item, cx)
			}),
		);

		let on_toggle = cx.callback_0_hash(
			&item,
			enclose!((item) move |cx| {
				toggle(&item, cx)
			}),
		);

		let item = item.get(cx);
		li((
			attr("data-id", item.id.to_string()),
			editing.0.then(|| classlist("editing")),
			item.is_done.then(|| classlist("completed")),
			if editing.0 {
				Either::A(input((
					reference(input_ref.setter()),
					classname("edit"),
					attr("value", item.title.to_owned()),
					attr(
						"onfocus",
						"this.selectionStart = this.selectionEnd = this.value.length;",
					),
					(on("keydown", on_keydown), on("blur", on_blur)),
				)))
			} else {
				Either::B(div((
					classname("view"),
					input((
						classname("toggle"),
						attr("type", "checkbox"),
						item.is_done.then(|| attr("checked", "")),
						on("click", on_toggle),
					)),
					label((item.title.to_owned(), on("click", on_click))),
					button((classname("destroy"), on("click", on_destroy))),
				)))
			},
		))
	})
}

fn update_title(
	item: &Var<TodoItem>,
	input_ref: &Mutable<Option<web_sys::Node>>,
	cx: &mut StatefulContext<WebSys, ReactiveExt<WebSys>>,
) {
	let item = item.get_once();
	let item = TodoItem {
		id: item.id.clone(),
		title: input_value(&input_ref.unwrap()),
		is_done: item.is_done,
	};

	cx.dispatch(TodoEdit(item));
}

fn destroy(item: &Var<TodoItem>, cx: &mut StatefulContext<WebSys, ReactiveExt<WebSys>>) {
	let item = item.get_once();
	cx.dispatch(TodoDelete(item.id.clone()));
}

fn toggle(item: &Var<TodoItem>, cx: &mut StatefulContext<WebSys, ReactiveExt<WebSys>>) {
	let item = item.get_once();
	cx.dispatch(TodoToggle(item.id.clone(), !item.is_done));
}
