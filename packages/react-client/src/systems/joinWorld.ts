import { Asset } from "lattice-engine/core";
import { Mut, Query, With } from "thyseus";

import { WorldJson } from "../components";
import { useClientStore } from "../store";

export function joinWorld(worlds: Query<Mut<Asset>, With<WorldJson>>) {
  const uri = useClientStore.getState().worldUri;

  for (const asset of worlds) {
    asset.uri = uri;
  }
}
