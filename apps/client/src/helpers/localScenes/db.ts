import { DBSchema, openDB } from "idb";
import { LocalScene } from "./types";

interface LocalSceneDB extends DBSchema {
  "local-scenes": {
    key: string;
    value: LocalScene;
  };
}

async function getDb() {
  const db = await openDB<LocalSceneDB>("wired", 1, {
    upgrade(db) {
      db.createObjectStore("local-scenes", {
        keyPath: "id",
      });
    },
  });

  return db;
}

export async function createLocalScene(localScene: LocalScene) {
  const db = await getDb();
  await db.add("local-scenes", localScene);
}

export async function deleteLocalScene(id: string) {
  const db = await getDb();
  await db.delete("local-scenes", id);
}

export async function getLocalScene(id: string) {
  const db = await getDb();
  const localScene = await db.get("local-scenes", id);
  return localScene;
}

export async function getLocalSceneIds() {
  const db = await getDb();
  const ids = await db.getAllKeys("local-scenes");
  return ids;
}

export async function mergeLocalScene(id: string, value: Partial<LocalScene>) {
  const db = await getDb();
  const localScene = await db.get("local-scenes", id);
  const newLocalScene = { ...localScene, ...value };
  await db.put("local-scenes", newLocalScene);
}
