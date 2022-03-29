import { DIDDataStore } from "@glazed/did-datastore";
import { useQuery } from "react-query";

import { BasicProfile } from "../models/BasicProfile/types";
import { ceramic, ceramicRead } from "../client";

const model = require("../models/BasicProfile/model.json");

export function useProfile(did: string) {
  async function merge(data: Partial<BasicProfile>) {
    const store = new DIDDataStore({ ceramic, model });
    await store.merge("basicProfile", data, { pin: true });
  }

  async function fetcher() {
    if (!did) return;
    const store = new DIDDataStore({ ceramic: ceramicRead, model });
    const profile = (await store.get("basicProfile", did)) as BasicProfile;
    return profile;
  }

  const { data } = useQuery(`basicProfile-${did}`, fetcher);

  return { profile: data, merge };
}
