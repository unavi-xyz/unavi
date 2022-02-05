import { useContext, useEffect, useState } from "react";
import { DIDDataStore } from "@glazed/did-datastore";

import { CeramicContext } from "..";

const model = require("../models/Worlds/model.json");

export function useWorlds(did: string) {
  const { ceramic, authenticated } = useContext(CeramicContext);

  const [worlds, setWorlds] = useState<string[]>();

  useEffect(() => {
    if (!authenticated) return;

    async function get() {
      const store = new DIDDataStore({ ceramic, model });
      const data = await store.get("worlds", did);
      setWorlds(data);
    }

    get();
  }, [authenticated, ceramic, did]);

  return worlds;
}
