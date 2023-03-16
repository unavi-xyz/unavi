import { useNode } from "../../hooks/useNode";
import { useNodeExtras } from "../../hooks/useNodeExtras";
import { useSpawn } from "../../hooks/useSpawn";
import { useSubscribe } from "../../hooks/useSubscribe";
import { useEditorStore } from "../../store";
import AddComponentButton, { COMPONENT_TYPE, ComponentType } from "./AddComponentButton";
import MeshComponent from "./mesh/MeshComponent";
import PhysicsComponent from "./PhysicsComponent";
import ScriptComponent from "./ScriptComponent";
import SpawnPointComponent from "./SpawnPointComponent";
import TransformComponent from "./TransformComponent";

export default function InspectMenu() {
  const isPlaying = useEditorStore((state) => state.isPlaying);
  const selectedId = useEditorStore((state) => state.selectedId);
  const node = useNode(selectedId);
  const name = useSubscribe(node, "Name");
  const mesh = useSubscribe(node, "Mesh");
  const extensions = useSubscribe(node, "Extensions") as any;
  const extras = useNodeExtras(node);
  const spawn = useSpawn();

  if (!node || !selectedId) return null;

  const availableComponents: ComponentType[] = [COMPONENT_TYPE.Script];

  if (!mesh) availableComponents.push(COMPONENT_TYPE.Mesh);
  if (!extensions?.OMI_collider) availableComponents.push(COMPONENT_TYPE.Physics);
  if (!extensions?.OMI_spawn_point && !spawn) availableComponents.push(COMPONENT_TYPE.SpawnPoint);

  return (
    <div className="pr-2 pb-4">
      <div className="flex w-full items-center justify-center pt-4">
        <input
          type="text"
          value={name ?? ""}
          disabled={isPlaying}
          onChange={(e) => node.setName(e.target.value)}
          className={`mx-10 w-full rounded-lg py-0.5 text-center text-2xl font-bold transition ${
            isPlaying ? "disabled:bg-white" : "hover:bg-neutral-200/80 focus:bg-neutral-200/80"
          }`}
        />
      </div>

      <div className="space-y-4 px-1">
        <TransformComponent node={node} />

        {mesh && <MeshComponent mesh={mesh} />}
        {extensions?.OMI_collider && <PhysicsComponent node={node} />}
        {extensions?.OMI_spawn_point && <SpawnPointComponent node={node} />}

        {extras?.scripts?.map(({ id }) => {
          return <ScriptComponent key={id} node={node} scriptId={id} />;
        })}

        {availableComponents.length > 0 && (
          <AddComponentButton
            availableComponents={availableComponents}
            node={node}
            extras={extras}
          />
        )}
      </div>
    </div>
  );
}
