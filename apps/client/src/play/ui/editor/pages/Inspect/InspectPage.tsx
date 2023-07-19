import { editNode } from "@/src/play/actions/editNode";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useTreeValue } from "../../hooks/useTreeValue";
import { getDisplayName } from "../../utils/getDisplayName";
import PanelPage from "../PanelPage";
import Rotation from "./Rotation";
import Scale from "./Scale";
import Translation from "./Translation";

interface Props {
  id: bigint;
}

export default function InspectPage({ id }: Props) {
  const name = useTreeValue(id, "name");

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

      <Translation id={id} />
      <Rotation id={id} />
      <Scale id={id} />
    </PanelPage>
  );
}
