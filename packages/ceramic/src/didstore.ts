import { DIDDataStore } from "@glazed/did-datastore";
import { ceramic } from "./client";
import { jsonArrayAdd, jsonArrayRemove } from "./json";

export async function addToArray(model: any, value: any) {
  if (!ceramic.did) return;
  const definition = Object.keys(model.definitions)[0];
  const store = new DIDDataStore({ ceramic, model });

  const oldArray = await store.get(definition);
  const newArray = jsonArrayAdd(oldArray, value);

  await store.set(definition, newArray, { pin: true });
}

export async function removeFromArray(model: any, value: any) {
  if (!ceramic.did) return;
  const definition = Object.keys(model.definitions)[0];
  const store = new DIDDataStore({ ceramic, model });

  const oldArray = await store.get(definition, ceramic.did.id);
  const newArray = jsonArrayRemove(oldArray, value);

  await store.set(definition, newArray, { pin: true });
}

export async function getArrayStore(model: any, did: string) {
  const definition = Object.keys(model.definitions)[0];
  const store = new DIDDataStore({ ceramic, model, id: did });

  const data = await store.get(definition);
  const deduplicated = Array.from(new Set(Object.values(data))) as string[];

  return deduplicated;
}
