import { Node } from "engine";

import { addNode } from "../../actions/AddNodeAction";
import { useEditorStore } from "../../store";

enum SpecialName {
  Spawn = "Spawn",
}

export default function SpecialsMenu() {
  const scene = useEditorStore((state) => state.engine?.scene);
  const spawnId = useEditorStore((state) => state.engine?.scene.spawnId);

  function addObject(name: SpecialName) {
    // Create node
    const selectedId = createNode(name);

    // Select new node
    useEditorStore.setState({ selectedId });
  }

  return (
    <div className="space-y-0.5 p-2">
      <button
        onClick={() => {
          // There can only be one spawn point
          if (scene?.spawnId) {
            useEditorStore.setState({ selectedId: scene.spawnId });
            return;
          }

          addObject(SpecialName.Spawn);
        }}
        className="flex w-full items-center space-x-4 rounded-lg px-4 py-0.5 transition hover:bg-primaryContainer hover:text-onPrimaryContainer"
      >
        <div>{SpecialName.Spawn}</div>
        <div className={`${spawnId ? "font-bold" : null}`}>
          {spawnId ? 1 : 0}/1
        </div>
      </button>
    </div>
  );
}

function createNode(name: SpecialName) {
  const { engine } = useEditorStore.getState();
  if (!engine) throw new Error("Engine not found");

  const node = new Node();
  node.name = name;

  addNode(node);

  engine.scene.spawnId = node.id;

  return node.id;
}
