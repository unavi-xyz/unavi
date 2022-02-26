import { loader } from "./client";

export async function merge(id: string, data: any, pin = false) {
  const doc = await loader.load(id);
  const newContent = Object.assign(doc.content, data);
  await doc.update(newContent, undefined, { pin });
}

export async function pin(id: string) {
  const doc = await loader.load(id);
  await doc.update(doc.content, undefined, { pin: true });
}

export async function unpin(id: string) {
  const doc = await loader.load(id);
  await doc.update(doc.content, undefined, { pin: false });
}
