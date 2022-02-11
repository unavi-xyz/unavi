import useSWR from "swr";
import { DIDDataStore } from "@glazed/did-datastore";
import { ceramicRead } from "..";

const model = require("../models/Rooms/model.json");

export function useRooms(did: string) {
  const ceramic = ceramicRead;

  async function fetcher() {
    if (!did) return;
    const store = new DIDDataStore({ ceramic, model });
    const data: string[] = Object.values(
      (await store.get("rooms", did)) as any
    );
    return data;
  }

  const { data } = useSWR(`rooms-${did}`, fetcher);

  return data;
}
