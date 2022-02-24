import { DIDDataStore } from "@glazed/did-datastore";

import { ceramic } from "../../constants";

const model = require("./model.json");

export async function joinSpace(streamId: string) {
  const store = new DIDDataStore({ ceramic, model });
  const oldSpaces = await store.get("spaces");
  const newSpaces = oldSpaces
    ? [...Object.values(oldSpaces), streamId]
    : [streamId];
  await store.set("spaces", newSpaces, { pin: true });
}

export async function leaveSpace(streamId: string) {
  if (!ceramic.did) return;
  const store = new DIDDataStore({ ceramic, model });
  const data = await store.get("spaces", ceramic.did.id);
  const newSpaces = Object.values(data).filter((id) => id !== streamId);
  await store.set("spaces", newSpaces, { pin: true });
}
