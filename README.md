# Skima

An experimental library for building web interfaces in Rust with full support of the Rust type system.

The goal is to provide a simple and clean API to build interfaces that should recemble a familliar React way, but better.

Minimal example:

```rust
use skima::web::prelude::*;

fn component() -> impl Markup {
  div((
    classname("myclass"),
    h1("Hello world")
  ))
} 
```

This is a simple funtion that returns some markup. No macros, no hidden magick, just Rust. The returned type `impl Markup` is hiding a complex type we build internally, which will be something like:

```ignore
Tag<"div", (ClassName, Tag<"html", &'static str>)>
```

As you can see, the library does not do any type erasure or heap allocation. Instead, we rely on Rust's [tuples](https://doc.rust-lang.org/rust-by-example/primitives/tuples.html) and native types like `Option` to build complex anonymous structures that represent your UI. We are not building a shapeless tree like other libraries do, instead we are building a statically-known tree and this enables some interesting optimizations:

1. We are not using any virtual DOM to make it work. Because we can guarantee that we return the same type every time, we can just rely on a pre-defined set of rules to update the tree.

2. We can know in a compile time what parts of the template are static to avoid reconcillation at the runtime.

## Features

### Components as functions

```rust
use skima::web::prelude::*;

#[derive(Eq, PartialEq)]
struct ButtonProps {
  value: usize,
  on_click: Option<Callback<dyn Fn(usize)>>,
}

fn my_button(props: ButtonProps) -> impl Markup {
  div((
    "Button",
    props.value.to_string(),
    props.on_click.map(|f| {
      on("click", move |_| f(props.value))
    })
  ))
}
```

### `Option` combinator

We implement the `Markup` trait for the native `Option` type. This allows to use Rust's native API to render something optionally:

```rust
use skima::web::prelude::*;

fn component(show_link: bool) -> impl Markup {
  div((
    show_link.then(|| a("I'm a link"))
  ))
}
```

*Any* markup may be wrapped with an Option, so you can toggle classes, attributes and anything else.

### Reactivity

```rust
use skima::web::prelude::*;

fn counter(name: String) -> impl Markup {
  reactive(|cx| {
    cx.with(0 as usize);

    let on_click = cx.callback_1(|cx, _| {
      let current = cx.get::<usize>();
      cx.set::<usize>(current + 1);
    });

    let current = cx.get::<usize>();

    div((
      current.to_string(), 
      on("click", on_click)
    ))
  })
}
```

### Variables with `observe`

Take a look: [observe](https://github.com/s-panferov/observe)

```rust 
use skima::web::prelude::*;
use observe::{Var, batch};

fn component() -> impl Markup {
  reactive(|cx| {
    let flag = cx.with_memo(|| Var::new(true));

    let text = if *flag.get(cx) {
      "True"
    } else {
      "False"
    };

    let on_click = cx.callback_1({
      let flag = flag.clone();
      move |_, ev| batch(|| flag.toggle())
    });

    div((
      text,
      on("click", on_click),
    ))
  })
}
```

### More features

* [Lists](./docs/List.md)

### Undocumented features

Those features do exist and work, but need some documentation and examples (TODO):

* Event handlers and callbacks
* Effects and cleanup in reactive components
* Static optimizations
* `Either` combinator
* Portals
* Bump allocation
* Routing helpers

## Limitations

* This library relies heavily on some of the nigtly features, so it works only on the `nigtly` Rust toolchain.

* Work in progress.