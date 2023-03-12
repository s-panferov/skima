use skima::action::Action;

#[derive(Debug)]
pub struct TodoCreate(pub String);

impl Action for TodoCreate {}
