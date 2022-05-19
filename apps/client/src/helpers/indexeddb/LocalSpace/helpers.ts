import { customAlphabet } from "nanoid";

import { LOCAL_SPACES_ID, STARTING_SCENE } from "./constants";
import { getLocalSpaceDb } from "./db";
import { LocalSpace } from "./types";

const nanoid = customAlphabet("0123456789", 12);

export async function getLocalSpace(id: string) {
  const db = await getLocalSpaceDb();
  const localScene = await db.get(LOCAL_SPACES_ID, id);
  return localScene;
}

export async function getLocalSpaces() {
  const db = await getLocalSpaceDb();
  const localSpaces = await db.getAll(LOCAL_SPACES_ID);
  return localSpaces;
}

export async function createLocalSpace(space: LocalSpace) {
  const db = await getLocalSpaceDb();
  await db.add(LOCAL_SPACES_ID, space);
  return space;
}

export async function createNewLocalSpace(options: Partial<LocalSpace> = {}) {
  const space: LocalSpace = {
    id: nanoid(),
    name: "My Scene",
    description: "",
    image: "/images/defaultSpaceImage.jpg",
    scene: STARTING_SCENE,
    ...options,
  };

  return createLocalSpace(space);
}

export async function deleteLocalSpace(id: string) {
  const db = await getLocalSpaceDb();
  await db.delete(LOCAL_SPACES_ID, id);
}

export async function updateLocalSpace(id: string, value: Partial<LocalSpace>) {
  const db = await getLocalSpaceDb();
  const space = await db.get(LOCAL_SPACES_ID, id);
  if (!space) throw new Error("Local space not found");

  const newLocalSpace = { ...space, ...value };
  await db.put(LOCAL_SPACES_ID, newLocalSpace);
}
