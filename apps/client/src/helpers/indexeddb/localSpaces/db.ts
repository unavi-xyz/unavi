import { DBSchema, openDB } from "idb";
import { LocalSpace } from "./types";

const NAME = "local-space";

interface LocalSpaceDB extends DBSchema {
  [NAME]: {
    key: string;
    value: LocalSpace;
  };
}

async function getDb() {
  const db = await openDB<LocalSpaceDB>("wired", 1, {
    upgrade(db) {
      db.createObjectStore(NAME, {
        keyPath: "id",
      });
    },
  });

  return db;
}

export async function createLocalSpace(localSpace: LocalSpace) {
  const db = await getDb();
  await db.add(NAME, localSpace);
}

export async function deleteLocalSpace(id: string) {
  const db = await getDb();
  await db.delete(NAME, id);
}

export async function getLocalSpace(id: string) {
  const db = await getDb();
  const localScene = await db.get(NAME, id);
  return localScene;
}

export async function getLocalSpaces() {
  const db = await getDb();
  const localSpaces = await db.getAll(NAME);
  return localSpaces;
}

export async function mergeLocalSpace(id: string, value: Partial<LocalSpace>) {
  const db = await getDb();
  const localSpace = await db.get(NAME, id);
  if (!localSpace) throw new Error("Local space not found");

  const newLocalSpace = { ...localSpace, ...value };
  await db.put(NAME, newLocalSpace);
}
