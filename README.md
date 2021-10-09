```rust
// main.rs
#[derive(Debug, Deserialize)]
struct ApiResponse {
  user_id: isize,
  id: isize,
  title: String,
  completed: bool,
}

let mut rt = Runtime::new("testdata/basic.js".to_owned());
let user_id = 5;
let value: ApiResponse = rt.call(5).await?;
```

```javascript
// testdata/basic.js
export async function hello(id) {
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
