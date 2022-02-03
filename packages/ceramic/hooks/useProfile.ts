import { useContext, useEffect, useState } from "react";
import { DIDDataStore } from "@glazed/did-datastore";

import { BasicProfile } from "../models/BasicProfile/types";
import { CeramicContext } from "..";

const model = require("../models/BasicProfile/model.json");

export function useProfile(did: string | undefined) {
  const { ceramic, authenticated } = useContext(CeramicContext);

  const [profile, setProfile] = useState<BasicProfile>();

  useEffect(() => {
    if (!authenticated || !did) return;

    async function get() {
      const store = new DIDDataStore({ ceramic, model });
      const data = await store.get("basicProfile", did);
      setProfile(data);
    }

    get();
  }, [authenticated, ceramic, did]);

  async function merge(data: any) {
    const store = new DIDDataStore({ ceramic, model });
    await store.merge("basicProfile", data);
  }

  return { profile, merge };
}
