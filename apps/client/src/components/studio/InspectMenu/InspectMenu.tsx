import { useAtomValue } from "jotai";

import { selectedAtom } from "../../../helpers/studio/atoms";
import EntityMenu from "./EntityMenu/EntityMenu";
import MaterialMenu from "./MaterialMenu/MaterialMenu";
import MenuBlock from "./MenuBlock";
import TransformMenu from "./TransformMenu";

const MATERIAL_TYPES = ["Box", "Sphere", "Cylinder"];
const NO_ENTITY_MENU_TYPES = ["Group", "Spawn"];

export default function InspectMenu() {
  const selected = useAtomValue(selectedAtom);

  if (!selected) return null;

  const hasMaterial = MATERIAL_TYPES.includes(selected.type);
  const hasEntityMenu = !NO_ENTITY_MENU_TYPES.includes(selected.type);

  return (
    <div key={selected.id} className="p-4 space-y-8 w-full">
      <div className="flex justify-center text-xl font-bold">
        {selected.name}
      </div>

      <MenuBlock menuId="Transform">
        <TransformMenu />
      </MenuBlock>

      {hasEntityMenu && (
        <MenuBlock menuId="Entity" title={selected.type}>
          <EntityMenu />
        </MenuBlock>
      )}

      {hasMaterial && (
        <MenuBlock menuId="Material">
          <MaterialMenu />
        </MenuBlock>
      )}
    </div>
  );
}
