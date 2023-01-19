# List support

The overall approach of the `skima` library is based on the fact that out interfaces are mostly static with some minimal degree of variablity and branching.

But, we still have adapt for scenarios where we need to display some user content. Most of the time it takes form of a *list*, so we have to define a way to display such lists.

Fortunately, this is pretty staitforward in `skima`:

```rust
use skima::web::prelude::*;

fn my_list() -> impl Markup {
  let list = [1, 2, 3];

  skima::list::list(
    list.into_iter().map(|v| (v, v)), 
    |value, key| { div(value.to_string()) }
  )
}
```

First argument of the list function accepts an iterator that returns `(KEY, VALUE)` pairs. The second argument accepts a function that returns a markup for a single item.