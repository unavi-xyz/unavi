import { DIDDataStore } from "@glazed/did-datastore";
import useSWR from "swr";
import { BasicProfile, ceramic, ceramicRead } from "..";

const model = require("../models/BasicProfile/model.json");

export function useProfile(did: string | undefined) {
  async function fetcher() {
    if (!did) return;
    const store = new DIDDataStore({ ceramic: ceramicRead, model });
    const data = (await store.get("basicProfile", did)) as BasicProfile;

    return data;
  }

  const { data } = useSWR(`basicProfile-${did}`, fetcher);

  async function merge(data: any) {
    const store = new DIDDataStore({ ceramic, model });
    await store.merge("basicProfile", data, { pin: true });
  }

  const imageHash = data?.image?.original.src.replace("ipfs://", "");
  const imageUrl = `https://ipfs.infura.io:5001/api/v0/cat?arg=${imageHash}`;

  return { profile: data, imageUrl, merge };
}
