import { SPAWN_TITLES, SpawnPointExtension } from "engine";
import { useId } from "react";

import { useSpawn } from "../../hooks/useSpawn";
import { useEditorStore } from "../../store";

const OBJECT_NAME = {
  Spawn: "Spawn",
  GLTF: "Import glTF",
} as const;

type ObjectName = (typeof OBJECT_NAME)[keyof typeof OBJECT_NAME];

export default function SpecialsMenu() {
  const id = useId();
  const spawn = useSpawn();

  const engine = useEditorStore((state) => state.engine);

  function addObject(name: ObjectName) {
    const { engine } = useEditorStore.getState();
    if (!engine) return;

    if (spawn) {
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
    <div className="space-y-0.5 p-2">
      <button
        onClick={() => addObject("Spawn")}
        className="flex w-full items-center space-x-2 whitespace-nowrap rounded-lg px-4 py-0.5 transition hover:bg-neutral-200"
      >
        <span>Spawn</span>
        <span className={spawn ? "font-bold" : ""}>({spawn ? 1 : 0}/1)</span>
      </button>

      <label onPointerUp={(e) => e.stopPropagation()} htmlFor={id}>
        <div className="flex w-full cursor-pointer items-center whitespace-nowrap rounded-lg px-4 py-0.5 transition hover:bg-neutral-200">
          Import glTF
        </div>
      </label>

      <input
        id={id}
        type="file"
        accept=".gltf,.glb"
        className="hidden"
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (!file) return;

          engine?.scene.addFile(file);
        }}
      />
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
      spawnPoint.title = SPAWN_TITLES.Default;
      node.setExtension(SpawnPointExtension.EXTENSION_NAME, spawnPoint);
      break;
    }
  }

  return id;
}
