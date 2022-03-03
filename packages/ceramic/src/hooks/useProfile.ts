import { DIDDataStore } from "@glazed/did-datastore";
import useSWR from "swr";

import { ceramic, ceramicRead } from "../client";
import { BasicProfile } from "../models/BasicProfile/types";

const model = require("../models/BasicProfile/model.json");

export function useProfile(did: string) {
  async function merge(data: any) {
    const store = new DIDDataStore({ ceramic, model });
    await store.merge("basicProfile", data, { pin: true });
  }

  async function fetcher() {
    if (!did) return;
    const store = new DIDDataStore({ ceramic: ceramicRead, model });
    const data = (await store.get("basicProfile", did)) as BasicProfile;

    const imageHash = data?.image?.original.src.replace("ipfs://", "");
    const imageUrl = imageHash
      ? `https://ipfs.io/ipfs/${imageHash}`
      : undefined;

    return { profile: data, imageUrl };
  }

  const { data } = useSWR(`basicProfile-${did}`, fetcher);

  return { profile: data?.profile, imageUrl: data?.imageUrl, merge };
}
