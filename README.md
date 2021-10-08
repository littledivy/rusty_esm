```javascript
export function verify() {
  return [
    "Hello",
    "World!",
    null,
  ];
}
```

```sh
$ target/debug/deno_embed
Array([
    String(
        "Hello",
    ),
    String(
        "World!",
    ),
    Null,
])
```
