import { useAtomValue } from "jotai";

import { selectedAtom } from "../../../../helpers/studio/atoms";
import MaterialPicker from "./MaterialPicker";
import MaterialProperties from "./MaterialProperties";

export default function MaterialMenu() {
  const selected = useAtomValue(selectedAtom);

  if (!selected) return null;

  return (
    <>
      <MaterialPicker selected={selected} />
      <MaterialProperties selected={selected} />
    </>
  );
}
