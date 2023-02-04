import { SPAWN_TITLES, SpawnPointExtension } from "engine";

import { useSpawn } from "../../hooks/useSpawn";
import { useEditorStore } from "../../store";

const OBJECT_NAME = {
  Spawn: "Spawn",
} as const;

type ObjectName = (typeof OBJECT_NAME)[keyof typeof OBJECT_NAME];

export default function SpecialsMenu() {
  const spawn = useSpawn();

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
      {Object.values(OBJECT_NAME).map((name) => (
        <button
          key={name}
          onClick={() => addObject(name)}
          className="flex w-full items-center space-x-2 whitespace-nowrap rounded-lg px-4 py-0.5 transition hover:bg-neutral-200"
        >
          <span>{name}</span>
          <span className={spawn ? "font-bold" : ""}>({spawn ? 1 : 0}/1)</span>
        </button>
      ))}
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
