use proc_macro::TokenStream;
use quote::quote;
use syn_rsx::{parse, Node, NodeName};

/// Converts HTML to `String`.
///
/// Values returned from braced blocks `{}` are expected to return something
/// that implements `Display`.
///
/// See [syn-rsx docs](https://docs.rs/syn-rsx/) for supported tags and syntax.
///
/// # Example
///
/// ```
/// use html_to_string_macro::html;
///
/// let world = "planet";
/// assert_eq!(html!(<div>"hello "{world}</div>), "<div>hello planet</div>");
/// ```
#[proc_macro]
pub fn html(tokens: TokenStream) -> TokenStream {
	match parse(tokens) {
		Ok(nodes) => {
			assert!(nodes.len() == 1, "A single node expected");
			let node = nodes.into_iter().next().unwrap();
			format_node(node)
		}
		Err(error) => error.to_compile_error(),
	}
	.into()
}

fn format_node(node: Node) -> proc_macro2::TokenStream {
	match node {
		Node::Element(el) => {
			let path = match el.name {
				NodeName::Path(path) => {
					quote!(#path)
				}
				_ => todo!(),
			};

			let mut nested = Vec::new();

			for attr in el.attributes {
				match attr {
					Node::Attribute(a) => {
						let name = match a.key {
							NodeName::Path(path) => {
								format!("{}", quote!(#path))
							}
							_ => unimplemented!(),
						};

						let value = match a.value {
							Some(expr) => {
								let expr = expr.as_ref();
								quote!(#expr)
							}
							None => {
								quote!("")
							}
						};

						match name.as_ref() {
							"class" => {
								nested.push(quote!(classlist(#value)));
							}
							_ => {
								let name = quote!(#name);
								nested.push(quote!(attr(#name, #value)))
							}
						}
					}
					_ => unimplemented!(),
				}
			}

			for child in el.children {
				nested.push(format_node(child))
			}

			quote! {#path((
				#(#nested),*
			))}
		}
		Node::Text(text) => {
			let value = text.value.as_ref();
			quote! { #value }
		}
		_ => todo!(),
	}
}
