import { Asset, Warehouse } from "lattice-engine/core";
import { Mut, Query, Res, With } from "thyseus";

import { useClientStore } from "../clientStore";
import { WorldJson } from "../components";

export function joinWorld(
  warehouse: Res<Mut<Warehouse>>,
  worlds: Query<Mut<Asset>, With<WorldJson>>
) {
  const uri = useClientStore.getState().worldUri;

  for (const asset of worlds) {
    asset.uri.write(uri, warehouse);
  }
}
