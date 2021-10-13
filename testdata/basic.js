// TODO: allow calling specific exports
export async function hello(id, id2) {
  const r1 = await fetch("https://jsonplaceholder.typicode.com/todos/" + id);
  const r2 = await fetch("https://jsonplaceholder.typicode.com/todos/" + id2);
  return await Promise.all([r1.json(), r2.json()]);
}
