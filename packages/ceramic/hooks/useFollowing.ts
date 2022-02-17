import { DIDDataStore } from "@glazed/did-datastore";
import useSWR from "swr";
import { ceramicRead } from "..";

const model = require("../models/Following/model.json");

export function useFollowing(did: string | undefined) {
  async function fetcher() {
    if (!did) return;
    const store = new DIDDataStore({ ceramic: ceramicRead, model });
    const data = (await store.get("following", did)) as string[];
    return data;
  }

  const { data } = useSWR(`following-${did}`, fetcher);

  return data;
}
