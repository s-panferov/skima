use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::tree::Tree;
use crate::web::{Markup, WebSys};
use crate::{console_log, render_subtree, subtree};

pub struct Root<M>
where
    M: Markup<WebSys>,
{
    pub(crate) tree: Tree<WebSys>,
    markup: M,
}

impl<M> Root<M>
where
    M: Markup<WebSys>,
{
    pub fn render(markup: M, container: HtmlElement) -> Root<M> {
        let tree = Tree::root(container.unchecked_into());
        console_log!("Render root subtree");

        render_subtree(&markup, &tree);
        Root { markup, tree }
    }

    pub fn ephemeral(markup: M) -> Root<M> {
        let tree = Tree::ephemeral_root();
        console_log!("Render root subtree");

        if !M::has_own_node() {
            panic!("Ephemeral Root requires markup to have an own tree node");
        }

        markup.render(&tree);

        Root { markup, tree }
    }

    pub fn element(&self) -> HtmlElement {
        self.tree.node().clone().unchecked_into()
    }

    pub fn update(&mut self, next: M) {
        let tree = if M::has_own_node() {
            self.tree.children.borrow().first().unwrap().0.clone()
        } else {
            self.tree.clone()
        };

        next.diff(&self.markup, &tree);
        self.markup = next;
    }
}

impl<M> Drop for Root<M>
where
    M: Markup<WebSys>,
{
    fn drop(&mut self) {
        let tree = subtree::<M, _>(&self.tree);

        self.markup.drop(&tree, false);
        self.tree
            .node()
            .unchecked_ref::<HtmlElement>()
            .set_inner_html("");
    }
}
