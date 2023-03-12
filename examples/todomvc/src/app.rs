use skima::tree::Tree;
use skima::web::prelude::*;
use skima::web::tree::tree;
use skima::web::WebSys;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, KeyboardEvent};

use crate::action::TodoCreate;

pub fn app_header() -> impl Markup {
	header((
		classname("header"),
		h1("todos"),
		input((
			classname("new-todo"),
			attr("placeholder", "What needs to be done?"),
			attr("autofocus", "true"),
			tree(|tree| on("keydown", on_keydown(tree.clone()))),
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
	ul((classname("todo-list"), todo_item()))
}

pub fn todo_item() -> impl Markup {
	li((
		classname("completed"),
		div((
			classname("view"),
			input((
				classname("toggle"),
				attr("type", "checkbox"),
				attr("checked", ""),
			)),
			label("Taste JavaScript"),
			button(classname("destroy")),
		)),
		input((
			classname("edit"),
			attr("value", "Create a TodoMVC template"),
		)),
	))
}

pub fn app() -> impl Markup {
	section((classname("todoapp"), app_header(), app_main()))
}

fn on_keydown(tree: Tree<WebSys>) -> impl Fn(Event) {
	move |ev: Event| {
		let ev: KeyboardEvent = ev.unchecked_into();
		match ev.key().as_ref() {
			"Enter" => tree.dispatch(Box::new(TodoCreate(value(ev)))),
			_ => {}
		}
	}
}

fn value(ev: KeyboardEvent) -> String {
	ev.target()
		.unwrap()
		.unchecked_into::<HtmlInputElement>()
		.value()
}
