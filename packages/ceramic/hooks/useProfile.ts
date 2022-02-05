import { DIDDataStore } from "@glazed/did-datastore";
import useSWR from "swr";
import { BasicProfile, ceramicRead } from "..";

const model = require("../models/BasicProfile/model.json");

export function useProfile(did: string | undefined) {
  const ceramic = ceramicRead;

  async function fetcher() {
    if (!did) return;
    const store = new DIDDataStore({ ceramic, model });
    const data = (await store.get("basicProfile", did)) as BasicProfile;
    return data;
  }

  const { data } = useSWR(`basicProfile-${did}`, fetcher);

  async function merge(data: any) {
    const store = new DIDDataStore({ ceramic, model });
    await store.merge("basicProfile", data);
  }

  return { profile: data, merge };
}
