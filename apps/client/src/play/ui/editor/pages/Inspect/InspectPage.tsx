import { useTreeValue } from "../../hooks/useTreeValue";
import { getDisplayName } from "../../utils/getDisplayName";
import PanelPage from "../PanelPage";
import { Name } from "./Name";
import Physics from "./Physics";
import Transform from "./Transform";

interface Props {
  id: bigint;
}

export default function InspectPage({ id }: Props) {
  const name = useTreeValue(id, "name");

  const displayName = getDisplayName(name, id);

  return (
    <PanelPage title={displayName}>
      <Name id={id} />
      <Transform id={id} />
      <Physics id={id} />
    </PanelPage>
  );
}
