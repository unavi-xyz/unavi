import { Asset } from "houseki/core";
import { Mut, Query, With } from "thyseus";

import { useClientStore } from "../clientStore";
import { WorldJson } from "../components";

export function joinWorld(worlds: Query<Mut<Asset>, With<WorldJson>>) {
  const uri = useClientStore.getState().worldUri;

  for (const asset of worlds) {
    asset.uri = uri;
  }
}
