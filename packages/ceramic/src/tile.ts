import { loader } from "./client";

export async function mergeTile(id: string, data: any, pin = false) {
  const doc = await loader.load(id);
  const newContent = Object.assign(doc.content, data);
  await doc.update(newContent, undefined, { pin });
}

export async function pinTile(id: string) {
  const doc = await loader.load(id);
  await doc.update(doc.content, undefined, { pin: true });
}

export async function unpinTile(id: string) {
  const doc = await loader.load(id);
  await doc.update(doc.content, undefined, { pin: false });
}
