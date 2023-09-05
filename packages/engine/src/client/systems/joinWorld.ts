import { Asset } from "lattice-engine/core";
import { Mut, Query, With } from "thyseus";

import { WorldJson } from "../components";
import { connectionStore } from "./connectToHost";

export function joinWorld(worlds: Query<Mut<Asset>, With<WorldJson>>) {
  const uri = connectionStore.get(connectionStore.worldUri);

  for (const asset of worlds) {
    asset.uri = uri;
  }
}
