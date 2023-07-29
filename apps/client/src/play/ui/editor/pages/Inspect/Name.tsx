import { editNode } from "@/src/play/actions/editNode";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useTreeValue } from "../../hooks/useTreeValue";
import { getDisplayName } from "../../utils/getDisplayName";

interface Props {
  id: bigint;
}

export default function Name({ id }: Props) {
  const name = useTreeValue(id, "name");
  const locked = useTreeValue(id, "locked");

  const displayName = getDisplayName(name, id);

  return (
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
  );
}
