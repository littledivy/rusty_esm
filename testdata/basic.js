// TODO: allow calling specific exports
export async function hello(id) {
// export function hello(id) {
  const r = await fetch("https://jsonplaceholder.typicode.com/todos/" + id);
  return await r.json();
  //return id;
}
