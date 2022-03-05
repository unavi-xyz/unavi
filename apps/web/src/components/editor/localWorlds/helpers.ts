import { openDB } from "idb";
import { LocalWorld } from "./types";

async function getDb() {
  const db = await openDB("wired", 1, {
    upgrade(db) {
      db.createObjectStore("local-worlds", {
        keyPath: "id",
      });
    },
  });

  return db;
}

export async function createLocalWorld(world: LocalWorld) {
  const db = await getDb();
  await db.add("local-worlds", world);
}

export async function getLocalWorld(id: string) {
  const db = await getDb();
  return (await db.get("local-worlds", id)) as LocalWorld;
}

export async function getLocalWorldIds() {
  const db = await getDb();
  return (await db.getAllKeys("local-worlds")) as string[];
}
