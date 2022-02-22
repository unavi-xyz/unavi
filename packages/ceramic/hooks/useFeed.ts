import { DIDDataStore } from "@glazed/did-datastore";
import useSWR from "swr";
import { ceramicRead } from "..";

const model = require("../models/Feed/model.json");

export function useFeed(did: string | undefined) {
  async function fetcher() {
    if (!did) return;
    const store = new DIDDataStore({ ceramic: ceramicRead, model });
    const data = (await store.get("feed", did)) as string[];

    return data;
  }

  const { data } = useSWR(`feed-${did}`, fetcher);

  return data;
}
