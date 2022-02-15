import CeramicClient from "@ceramicnetwork/http-client";
import { DIDDataStore } from "@glazed/did-datastore";

const model = require("./model.json");

export async function addRoom(streamId: string, ceramic: CeramicClient) {
  const store = new DIDDataStore({ ceramic, model });
  const oldRooms = await store.get("rooms");
  const newRooms = oldRooms
    ? [...Object.values(oldRooms), streamId]
    : [streamId];
  await store.set("rooms", newRooms, { pin: true });
}

export async function removeRoom(
  streamId: string,
  userId: string,
  ceramic: CeramicClient
) {
  const store = new DIDDataStore({ ceramic, model });
  const data = (await store.get("rooms", userId)) as string[];
  const newRooms = data.filter((id) => id !== streamId);
  await store.set("rooms", newRooms, { pin: true });
}
