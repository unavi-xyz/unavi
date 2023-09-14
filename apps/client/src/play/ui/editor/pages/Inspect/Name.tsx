import { editNode } from "@/src/play/actions/editNode";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useNodeValue } from "../../hooks/useNodeValue";
import { getDisplayName } from "../../utils/getDisplayName";

interface Props {
  entityId: bigint;
}

export default function Name({ entityId }: Props) {
  const id = useNodeValue(entityId, "id");
  const name = useNodeValue(entityId, "name");
  const locked = useNodeValue(entityId, "locked");

  const displayName = getDisplayName(name, entityId);

  return (
    <TextFieldDark
      label="Name"
      value={name}
      disabled={locked}
      placeholder={displayName}
      onChange={(e) => {
        if (!id || !name) {
          return;
        }

        editNode(id, {
          name: e.target.value,
        });
      }}
    />
  );
}
