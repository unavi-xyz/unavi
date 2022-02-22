import CeramicClient from "@ceramicnetwork/http-client";
import { DIDDataStore } from "@glazed/did-datastore";

const model = require("./model.json");

export async function follow(targetId: string, ceramic: CeramicClient) {
  const store = new DIDDataStore({ ceramic, model });
  const oldFollowing = await store.get("following");
  const newFollowing = oldFollowing
    ? [targetId, ...Object.values(oldFollowing)]
    : [targetId];
  await store.set("following", newFollowing, { pin: true });
}

export async function unfollow(targetId: string, ceramic: CeramicClient) {
  const store = new DIDDataStore({ ceramic, model });
  const data = await store.get("following");
  if (!data) return;
  const newFollowing = Object.values(data).filter((id) => id !== targetId);
  await store.set("following", newFollowing, { pin: true });
}
