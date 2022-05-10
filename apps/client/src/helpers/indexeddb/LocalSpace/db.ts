import { DBSchema, openDB } from "idb";

import { LOCAL_SPACES_ID } from "./constants";
import { LocalSpace } from "./types";

interface LocalSpaceDB extends DBSchema {
  [LOCAL_SPACES_ID]: {
    key: string;
    value: LocalSpace;
  };
}

export async function getLocalSpaceDb() {
  const db = await openDB<LocalSpaceDB>(LOCAL_SPACES_ID, 1, {
    upgrade(db) {
      db.createObjectStore(LOCAL_SPACES_ID, {
        keyPath: "id",
      });
    },
  });

  return db;
}
