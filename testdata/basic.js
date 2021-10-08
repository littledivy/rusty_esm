// TODO: allow calling specific exports
export async function verify() {
  const r = await fetch("https://jsonplaceholder.typicode.com/todos/5");
  return await r.json();
}
