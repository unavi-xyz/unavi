import { useTreeValue } from "../../hooks/useTreeValue";
import { getDisplayName } from "../../utils/getDisplayName";
import PanelPage from "../PanelPage";
import AddComponent from "./AddComponent";
import Name from "./Name";
import Physics from "./Physics";
import Transform from "./Transform";

interface Props {
  entityId: bigint;
}

export default function InspectPage({ entityId }: Props) {
  const name = useTreeValue(entityId, "name");

  const displayName = getDisplayName(name, entityId);

  return (
    <PanelPage title={displayName}>
      <Name entityId={entityId} />
      <Transform entityId={entityId} />
      <Physics entityId={entityId} />
      <AddComponent entityId={entityId} />
    </PanelPage>
  );
}
