import { SpawnPointExtension } from "engine";

import { useSpawnPoint } from "../../hooks/useExtension";
import { useExtensionAttribute } from "../../hooks/useExtensionAttribute";
import { useNode } from "../../hooks/useNode";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  nodeId: string;
}

export default function SpawnPointComponent({ nodeId }: Props) {
  const node = useNode(nodeId);
  const spawnPoint = useSpawnPoint(node);
  const title = useExtensionAttribute(spawnPoint, "title");

  if (!spawnPoint) return null;

  return (
    <ComponentMenu
      title="Spawn Point"
      onRemove={() => {
        node?.setExtension(SpawnPointExtension.EXTENSION_NAME, null);
      }}
    >
      <MenuRows titles={["Title"]}>
        <div className="w-full cursor-not-allowed select-none rounded border border-neutral-300 bg-neutral-50 pl-1.5">
          {title}
        </div>
      </MenuRows>
    </ComponentMenu>
  );
}
