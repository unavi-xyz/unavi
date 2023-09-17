import { editNode, SyncedNode } from "@unavi/engine";

import { DeepReadonly } from "@/src/play/utils/types";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { getDisplayName } from "../../utils/getDisplayName";

interface Props {
  node: DeepReadonly<SyncedNode>;
}

export default function Name({ node }: Props) {
  const displayName = getDisplayName(node.name, node.id);

  return (
    <TextFieldDark
      label="Name"
      value={node.name}
      disabled={node.locked}
      placeholder={displayName}
      onChange={(e) => {
        editNode(node.id, {
          name: e.target.value,
        });
      }}
    />
  );
}
