import { customAlphabet } from "nanoid";
import { DEFAULT_SCENE } from "scene";

import { getLocalSpaceDb, LOCAL_SPACES } from "./db";
import { LocalSpace } from "./types";

const nanoid = customAlphabet("0123456789", 12);

export async function getLocalSpace(id: string) {
  const db = await getLocalSpaceDb();
  const localScene = await db.get(LOCAL_SPACES, id);
  return localScene;
}

export async function getLocalSpaces() {
  const db = await getLocalSpaceDb();
  const localSpaces = await db.getAll(LOCAL_SPACES);
  return localSpaces;
}

export async function createLocalSpace(localSpace: LocalSpace) {
  const db = await getLocalSpaceDb();
  await db.add(LOCAL_SPACES, localSpace);
  return localSpace;
}

export async function createNewLocalSpace(options: Partial<LocalSpace> = {}) {
  const localSpace: LocalSpace = {
    id: nanoid(),
    name: "My Space",
    description: "",
    image: "",
    scene: DEFAULT_SCENE,
    ...options,
  };

  return createLocalSpace(localSpace);
}

export async function deleteLocalSpace(id: string) {
  const db = await getLocalSpaceDb();
  await db.delete(LOCAL_SPACES, id);
}

export async function updateLocalSpace(id: string, value: Partial<LocalSpace>) {
  const db = await getLocalSpaceDb();
  const localSpace = await db.get(LOCAL_SPACES, id);
  if (!localSpace) throw new Error("Local space not found");

  const newLocalSpace = { ...localSpace, ...value };
  await db.put(LOCAL_SPACES, newLocalSpace);
}
