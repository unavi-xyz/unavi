import { DBSchema, openDB } from "idb";
import { LocalSpace } from "./types";

export const LOCAL_SPACES = "wired/local-spaces";

interface LocalSpaceDB extends DBSchema {
  [LOCAL_SPACES]: {
    key: string;
    value: LocalSpace;
  };
}

export async function getLocalSpaceDb() {
  const db = await openDB<LocalSpaceDB>(LOCAL_SPACES, 1, {
    upgrade(db) {
      db.createObjectStore(LOCAL_SPACES, {
        keyPath: "id",
      });
    },
  });

  return db;
}
