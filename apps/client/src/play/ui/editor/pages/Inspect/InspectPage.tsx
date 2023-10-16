import { syncedStore } from "@unavi/engine";
import { useSnapshot } from "valtio";

import { getDisplayName } from "../../utils/getDisplayName";
import PanelPage from "../PanelPage";
import AddComponent from "./AddComponent";
import Name from "./Name";
import Physics from "./Physics";
import Transform from "./Transform";

interface Props {
  id: string;
}

export default function InspectPage({ id }: Props) {
  const snap = useSnapshot(syncedStore);
  const node = id ? snap.nodes[id] : undefined;

  if (!node || !id) {
    return null;
  }

  const displayName = getDisplayName(node.name, id);

  return (
    <PanelPage title={displayName}>
      <Name node={node} />
      <Transform node={node} />
      <Physics node={node} />
      <AddComponent node={node} />
    </PanelPage>
  );
}
