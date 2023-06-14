import { WorldJson } from "@unavi/engine";
import { Asset } from "lattice-engine/core";
import { Mut, Query, With } from "thyseus";

import { config } from "../config";

export function setWorld(worlds: Query<Mut<Asset>, With<WorldJson>>) {
  for (const asset of worlds) {
    asset.uri = config.worldUri;
  }
}
