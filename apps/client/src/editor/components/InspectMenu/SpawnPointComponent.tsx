import { Node } from "@gltf-transform/core";

import { useSpawnPoint } from "../../hooks/useExtension";
import { useSubscribe } from "../../hooks/useSubscribe";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  node: Node;
}

export default function SpawnPointComponent({ node }: Props) {
  const spawnPoint = useSpawnPoint(node);
  const title = useSubscribe(spawnPoint, "Title");

  if (!spawnPoint) return null;

  return (
    <ComponentMenu
      title="Spawn Point"
      onRemove={() => {
        spawnPoint.dispose();
      }}
    >
      <MenuRows titles={["Title"]}>
        <div className="w-full cursor-default select-none rounded border border-neutral-300 bg-neutral-50 pl-1.5">
          {title}
        </div>
      </MenuRows>
    </ComponentMenu>
  );
}
