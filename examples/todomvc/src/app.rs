use skima::list::list;
use skima::tree::Tree;
use skima::web::prelude::*;
use skima::web::{hook, WebSys};
use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent};

use crate::action::TodoCreate;
use crate::helpers::input_event_value;
use crate::todo::todo_item;
use crate::TodoItems;

pub fn app_header() -> impl Markup {
	header((
		classname("header"),
		h1("todos"),
		input((
			classname("new-todo"),
			attr("placeholder", "What needs to be done?"),
			attr("autofocus", "true"),
			hook(|tree| on("keydown", on_keydown(tree.clone()))),
		)),
	))
}

pub fn app_main() -> impl Markup {
	section((
		classname("main"),
		input((
			classname("toggle-all"),
			attr("id", "toggle-all"),
			attr("type", "checkbox"),
		)),
		label((attr("for", "toggle-all"), "Mark all as complete")),
		todo_list(),
	))
}

pub fn todo_list() -> impl Markup {
	reactive(|cx| {
		let todos_var = cx.env::<TodoItems>();
		let todos_vec = todos_var.get(cx);
		let todos_iter = todos_vec.clone().into_iter();

		ul((
			classname("todo-list"),
			list(todos_iter, |t, _| todo_item(t.clone())),
		))
	})
}

pub fn app() -> impl Markup {
	section((classname("todoapp"), app_header(), app_main()))
}

fn on_keydown(tree: Tree<WebSys>) -> impl Fn(Event) {
	move |ev: Event| {
		let ev: KeyboardEvent = ev.unchecked_into();
		match ev.key().as_ref() {
			"Enter" => tree.dispatch(Box::new(TodoCreate(input_event_value(&ev)))),
			_ => {}
		}
	}
}
