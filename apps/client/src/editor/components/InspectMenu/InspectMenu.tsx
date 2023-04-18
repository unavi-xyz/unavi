import { EXTENSION_NAME } from "@unavi/gltf-extensions";

import { useNode } from "../../hooks/useNode";
import { useNodeExtras } from "../../hooks/useNodeExtras";
import { useSpawn } from "../../hooks/useSpawn";
import { useSubscribe } from "../../hooks/useSubscribe";
import { useEditor } from "../Editor";
import AddComponentButton, { COMPONENT_TYPE, ComponentType } from "./AddComponentButton";
import AudioComponent from "./AudioComponent";
import AvatarComponent from "./AvatarComponent";
import MeshComponent from "./mesh/MeshComponent";
import PhysicsComponent from "./PhysicsComponent";
import ScriptComponent from "./ScriptComponent";
import SpawnPointComponent from "./SpawnPointComponent";
import TransformComponent from "./TransformComponent";

interface Props {
  projectId: string;
}

export default function InspectMenu({ projectId }: Props) {
  const { selectedId } = useEditor();

  const { mode } = useEditor();
  const node = useNode(selectedId);
  const name = useSubscribe(node, "Name");
  const mesh = useSubscribe(node, "Mesh");
  const extensions = useSubscribe(node, "Extensions") ?? [];
  const extras = useNodeExtras(node);
  const spawn = useSpawn();

  if (!node || !selectedId) return null;

  const availableComponents: ComponentType[] = [COMPONENT_TYPE.Script];

  const hasAudioEmitter = extensions.find((ext) => ext.extensionName === EXTENSION_NAME.Audio);
  const hasAvatar = extensions.find((ext) => ext.extensionName === EXTENSION_NAME.Avatar);
  const hasCollider = extensions.find((ext) => ext.extensionName === EXTENSION_NAME.Collider);
  const hasSpawnPoint = extensions.find((ext) => ext.extensionName === EXTENSION_NAME.SpawnPoint);

  if (!mesh) availableComponents.push(COMPONENT_TYPE.Mesh);
  if (!hasAudioEmitter) availableComponents.push(COMPONENT_TYPE.Audio);
  if (!hasAvatar) availableComponents.push(COMPONENT_TYPE.Avatar);
  if (!hasCollider) availableComponents.push(COMPONENT_TYPE.Physics);
  if (!spawn) availableComponents.push(COMPONENT_TYPE.SpawnPoint);

  return (
    <div className="h-full pb-12">
      <div className="flex w-full items-center justify-center pt-4">
        <input
          type="text"
          value={name ?? ""}
          disabled={mode === "play"}
          onChange={(e) => node.setName(e.target.value)}
          className={`mx-10 w-full rounded-lg py-0.5 text-center text-2xl font-bold transition ${
            mode === "play"
              ? "disabled:bg-white"
              : "hover:bg-neutral-200/80 focus:bg-neutral-200/80"
          }`}
        />
      </div>

      <div className="h-full space-y-4 overflow-y-auto px-1 pb-2">
        <TransformComponent node={node} />

        {mesh && <MeshComponent mesh={mesh} />}

        {hasAudioEmitter && <AudioComponent projectId={projectId} node={node} />}
        {hasAvatar && <AvatarComponent projectId={projectId} node={node} />}
        {hasCollider && <PhysicsComponent node={node} />}
        {hasSpawnPoint && <SpawnPointComponent node={node} />}

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
