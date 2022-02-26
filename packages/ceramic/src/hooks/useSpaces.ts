import useSWR from "swr";
import { DIDDataStore } from "@glazed/did-datastore";
import { ceramic } from "../client";

const model = require("../models/Spaces/model.json");

export function useSpaces(did: string) {
  async function fetcher() {
    if (!did) return;
    const store = new DIDDataStore({ ceramic, model });
    const data: string[] = Object.values(
      (await store.get("spaces", did)) as any
    );
    const deduplicated = Array.from(new Set(data));
    return deduplicated;
  }

  const { data } = useSWR(`spaces-${did}`, fetcher);

  return data;
}
