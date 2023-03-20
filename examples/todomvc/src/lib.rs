use action::{TodoCreate, TodoDelete, TodoEdit, TodoToggle};
use helpers::ObservedMap;
use observe::{batch, Var};
use skima::action::Action;
use skima::web::dispatch::{capture, provide, ActionResult};
use skima::web::root::Root;
use skima::web::Callback;
use todo::TodoItem;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

mod action;
mod app;
mod helpers;
mod todo;

pub type TodoItems = Var<ObservedMap<Uuid, Var<TodoItem>>>;

#[wasm_bindgen(start)]
fn main() -> Result<(), JsError> {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));

	let el = web_sys::window()
		.unwrap()
		.document()
		.unwrap()
		.body()
		.unwrap();

	let todos = TodoItems::default();

	let on_action = Callback::new({
		let todos = todos.clone();
		move |action: Box<dyn Action>| {
			if action.is::<TodoCreate>() {
				let create = action.downcast::<TodoCreate>().unwrap();
				batch(|| {
					let id = Uuid::new_v4();
					todos.update(|v| {
						v.insert(
							id,
							Var::new(TodoItem {
								id,
								title: create.0,
								is_done: false,
							}),
						);
					})
				})
			} else if action.is::<TodoEdit>() {
				let edit = action.downcast::<TodoEdit>().unwrap();
				batch(|| {
					todos.update(|v| {
						v.entry(edit.0.id.clone()).and_modify(|e| {
							e.update(|todo| {
								todo.title = edit.0.title;
								todo.is_done = edit.0.is_done;
							})
						});
					})
				})
			} else if action.is::<TodoDelete>() {
				let delete = action.downcast::<TodoDelete>().unwrap();
				batch(|| {
					todos.update(|todos| {
						todos.shift_remove_entry(&delete.0);
					})
				})
			} else if action.is::<TodoToggle>() {
				let toggle = action.downcast::<TodoToggle>().unwrap();
				batch(|| {
					todos.update(|todos| {
						todos
							.get(&toggle.0)
							.unwrap()
							.update(|todo| todo.is_done = toggle.1);
					})
				})
			}
			ActionResult::Stop
		}
	});

	let root = Root::render(capture(provide(todos, app::app()), on_action.to_dyn()), el);

	// Otherwise our markup will be dropped
	std::mem::forget(root);
	Ok(())
}
