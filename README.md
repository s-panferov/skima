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

As you can see, the library does not do any type erasure, we are not building a shapeless tree like other libraries do. Instead we are building a statically-known type and this enables some interesting optimizations:

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

fn button(props: ButtonProps) -> impl Markup {
  div((
    "Button",
    props.value.to_string(),
    on_if(props.on_click, "click", move |f, _| f(props.value)),
  ))
}
```

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

### More

Those features do exist and work, but need some documentation and examples (TODO):

* Event handlers and callbacks
* Effects and cleanup
* Static optimizations
* Combinators (Option, Tuple)
* Portals
* Bump allocation
* Routing helpers
