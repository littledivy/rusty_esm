## rusty_esm

This is an example showcasing a common use case for embedding Deno - calling JS
module exports from Rust. The rest is pretty self explainatory:

```rust
// main.rs
#[derive(Debug, Deserialize)]
struct ApiResponse {
  user_id: isize,
  id: isize,
  title: String,
  completed: bool,
}

let mut rt = Runtime::new("testdata/basic.js").await?;
let id = 5;
let value: ApiResponse = rt.call("getStuff", &[id]).await?;
```

```javascript
// testdata/basic.js
export async function getStuff(id) {
  const r = await fetch("https://jsonplaceholder.typicode.com/todos/" + id);
  return await r.json();
}
```

```sh
$ target/debug/deno_embed
ApiResponse {
    user_id: 0,
    id: 5,
    title: "laboriosam mollitia et enim quasi adipisci quia provident illum",
    completed: false,
}
```

### Benchmarks

Measured for `Runtime::call` in example scenarios. See `src/main.rs`.

```shell
test tests::bench_call         ... bench:       4,429 ns/iter (+/- 1,139)
test tests::bench_call_promise ... bench:       4,609 ns/iter (+/- 306)
```

#### License

MIT
