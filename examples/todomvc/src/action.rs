use skima::action::Action;
use uuid::Uuid;

use crate::todo::TodoItem;

#[derive(Debug)]
pub struct TodoCreate(pub String);

impl Action for TodoCreate {}

#[derive(Debug)]
pub struct TodoEdit(pub TodoItem);
impl Action for TodoEdit {}
#[derive(Debug)]

pub struct TodoToggle(pub Uuid, pub bool);
impl Action for TodoToggle {}

#[derive(Debug)]
pub struct TodoDelete(pub Uuid);
impl Action for TodoDelete {}
