import CeramicClient from "@ceramicnetwork/http-client";
import { DIDDataStore } from "@glazed/did-datastore";

const model = require("./model.json");

export async function addWorld(streamId: string, ceramic: CeramicClient) {
  const store = new DIDDataStore({ ceramic, model });
  const oldWorlds = await store.get("worlds");
  const newWorlds = oldWorlds
    ? [...Object.values(oldWorlds), streamId]
    : [streamId];
  await store.set("worlds", newWorlds, { pin: true });
}

export async function removeWorld(
  streamId: string,
  userId: string,
  ceramic: CeramicClient
) {
  const store = new DIDDataStore({ ceramic, model });
  const data = await store.get("worlds", userId);
  const newWorlds = Object.values(data).filter((id) => id !== streamId);
  await store.set("worlds", newWorlds, { pin: true });
}
