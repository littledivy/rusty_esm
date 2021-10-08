```javascript
// basic.js
export async function verify() {
  const r = await fetch("https://jsonplaceholder.typicode.com/todos/5");
  return await r.json();
}
```

```sh
$ target/debug/deno_embed
Object({
    "userId": Number(
        1,
    ),
    "id": Number(
        5,
    ),
    "title": String(
        "laboriosam mollitia et enim quasi adipisci quia provident illum",
    ),
    "completed": Bool(
        false,
    ),
})
```

### TODO

- Support passing arguments (`serde_v8`).
- Support calling named exports and not just of current (`verify`)
