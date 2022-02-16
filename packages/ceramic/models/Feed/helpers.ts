import CeramicClient from "@ceramicnetwork/http-client";
import { DIDDataStore } from "@glazed/did-datastore";

const model = require("./model.json");

export async function addStatus(streamId: string, ceramic: CeramicClient) {
  const store = new DIDDataStore({ ceramic, model });
  const oldFeed = await store.get("feed");
  const newFeed = oldFeed ? [streamId, ...Object.values(oldFeed)] : [streamId];
  await store.set("feed", newFeed, { pin: true });
}

export async function removeStatus(
  streamId: string,
  userId: string,
  ceramic: CeramicClient
) {
  const store = new DIDDataStore({ ceramic, model });
  const data = await store.get("feed", userId);
  const newFeed = Object.values(data).filter((id) => id !== streamId);
  await store.set("feed", newFeed, { pin: true });
}
