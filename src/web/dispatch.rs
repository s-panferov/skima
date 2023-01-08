use std::any::Any;
use std::rc::Rc;

use crate::tree::Tree;
use crate::web::{Markup, WebSys};

use downcast_rs::Downcast;

pub trait Action: Downcast + std::fmt::Debug {}

downcast_rs::impl_downcast!(Action);

pub struct Dispatcher {
    func: Rc<dyn Fn(Box<dyn Action>)>,
}

impl Clone for Dispatcher {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
        }
    }
}

impl Dispatcher {
    pub fn dispatch<T: Action>(&self, op: T) {
        (self.func)(Box::new(op) as Box<dyn Action>)
    }

    pub fn new<F>(func: F) -> Self
    where
        F: Fn(Box<dyn Action>) + 'static,
    {
        Dispatcher {
            func: Rc::new(func),
        }
    }
}

pub trait DispatcherExt {
    fn dispatch<T: Action>(&self, action: T);
}

impl DispatcherExt for Tree<WebSys> {
    fn dispatch<T: Action>(&self, action: T) {
        if let Some(dispatcher) = self.try_data::<Dispatcher>() {
            dispatcher.dispatch(action)
        } else {
            if let Some(parent) = &self.parent {
                parent.dispatch(action)
            }
        }
    }
}

struct Provide<T, M> {
    data: Rc<T>,
    markup: M,
}

impl<T: Any + 'static, M: Markup<WebSys>> Markup<WebSys> for Provide<T, M> {
    fn has_own_node() -> bool {
        M::has_own_node()
    }

    fn render(&self, tree: &crate::tree::Tree<WebSys>) {
        tree.set_data(self.data.clone());
        self.markup.render(tree);
    }

    fn diff(&self, prev: &Self, tree: &crate::tree::Tree<WebSys>) {
        self.markup.diff(&prev.markup, &tree);
    }

    fn drop(&self, tree: &crate::tree::Tree<WebSys>, should_unmount: bool) {
        tree.remove_data::<T>();
        self.markup.drop(tree, should_unmount)
    }
}

pub fn provide<T: Any + 'static>(value: T, markup: impl Markup) -> impl Markup {
    Provide {
        data: Rc::new(value),
        markup,
    }
}
