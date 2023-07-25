import { editNode } from "@/src/play/actions/editNode";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useTreeValue } from "../../hooks/useTreeValue";
import { getDisplayName } from "../../utils/getDisplayName";
import PanelPage from "../PanelPage";
import Physics from "./Physics";
import Transform from "./Transform";

interface Props {
  id: bigint;
}

export default function InspectPage({ id }: Props) {
  const name = useTreeValue(id, "name");
  const locked = useTreeValue(id, "locked");

  const displayName = getDisplayName(name, id);

  return (
    <PanelPage title={displayName}>
      <TextFieldDark
        label="Name"
        value={name}
        disabled={locked}
        placeholder={displayName}
        onChange={(e) => {
          if (!name) {
            return;
          }

          editNode({
            name: e.target.value,
            target: name,
          });
        }}
      />

      <Transform id={id} />
      <Physics id={id} />
    </PanelPage>
  );
}