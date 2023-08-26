import { Asset, Warehouse } from "lattice-engine/core";
import { Mut, Query, Res, With } from "thyseus";

import { WorldJson } from "../components";
import { connectionStore } from "./connectToHost";

export function joinWorld(
  warehouse: Res<Mut<Warehouse>>,
  worlds: Query<Mut<Asset>, With<WorldJson>>
) {
  const uri = connectionStore.get(connectionStore.worldUri);

  for (const asset of worlds) {
    asset.uri.write(uri, warehouse);
  }
}
