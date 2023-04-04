import { SPAWN_TITLE } from "@wired-labs/gltf-extensions";

import { useEditorStore } from "@/app/editor/[id]/store";

import { DropdownItem } from "../../../ui/DropdownMenu";
import { useSpawn } from "../../hooks/useSpawn";

const OBJECT_NAME = {
  Spawn: "Spawn",
  GLTF: "Import glTF",
} as const;

type ObjectName = (typeof OBJECT_NAME)[keyof typeof OBJECT_NAME];

export default function SpecialsMenu() {
  const spawn = useSpawn();

  function addObject(name: ObjectName) {
    const { engine } = useEditorStore.getState();
    if (!engine) return;

    if (name === "Spawn" && spawn) {
      // Select existing node
      const spawnId = engine.scene.node.getId(spawn);
      if (!spawnId) throw new Error("Spawn node not found");

      useEditorStore.setState({ selectedId: spawnId });
      return;
    }

    // Create node
    const id = createNode(name);

    // Select new node
    useEditorStore.setState({ selectedId: id });
  }

  return (
    <div className="py-2">
      <DropdownItem
        onClick={() => addObject("Spawn")}
        className="flex w-full cursor-default items-center space-x-2 whitespace-nowrap px-6 outline-none focus:bg-neutral-200 active:opacity-80"
      >
        <div>Spawn</div>
        <div className={spawn ? "font-bold" : ""}>({spawn ? 1 : 0}/1)</div>
      </DropdownItem>

      <DropdownItem className="outline-none focus:bg-neutral-200">
        <label
          onClick={(e) => e.stopPropagation()}
          className="flex w-full items-center whitespace-nowrap px-6 outline-none focus:bg-neutral-200 active:opacity-80"
        >
          <div>Import glTF</div>

          <input
            type="file"
            accept=".gltf,.glb"
            className="hidden"
            onChange={async (e) => {
              const { engine } = useEditorStore.getState();
              if (!engine) return;

              const file = e.target.files?.[0];
              if (!file) return;

              await engine.scene.addFile(file);
            }}
          />
        </label>
      </DropdownItem>
    </div>
  );
}

function createNode(name: ObjectName) {
  const { engine } = useEditorStore.getState();
  if (!engine) return;

  const { id, object: node } = engine.scene.node.create();

  node.setName(name);

  switch (name) {
    case OBJECT_NAME.Spawn: {
      const spawnPoint = engine.scene.extensions.spawn.createSpawnPoint();
      spawnPoint.setTitle(SPAWN_TITLE.Default);
      node.setExtension(spawnPoint.extensionName, spawnPoint);
      break;
    }
  }

  return id;
}
