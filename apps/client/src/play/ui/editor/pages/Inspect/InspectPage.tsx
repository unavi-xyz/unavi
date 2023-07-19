import { useSceneStore } from "@unavi/react-client";

import { editNode } from "@/src/play/actions/editNode";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useTreeItem } from "../../hooks/useTreeItem";
import { getDisplayName } from "../../utils/getDisplayName";
import PanelPage from "../PanelPage";

interface Props {
  id: bigint;
}

export default function InspectPage({ id }: Props) {
  const name = useSceneStore((state) => state.items.get(id)?.name);
  const item = useTreeItem(id);

  if (!item) {
    return null;
  }

  const displayName = getDisplayName(name, id);

  return (
    <PanelPage title={displayName}>
      <TextFieldDark
        label="Name"
        value={name}
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
    </PanelPage>
  );
}
