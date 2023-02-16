import { ColliderExtension, SPAWN_TITLES } from "engine";
import { nanoid } from "nanoid";
import { useState } from "react";

import Button from "../../../ui/Button";
import {
  DropdownContent,
  DropdownItem,
  DropdownMenu,
  DropdownMenuItemProps,
  DropdownTrigger,
} from "../../../ui/DropdownMenu";
import { useNode } from "../../hooks/useNode";
import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { useSpawn } from "../../hooks/useSpawn";
import { useEditorStore } from "../../store";
import MeshComponent from "./mesh/MeshComponent";
import PhysicsComponent from "./PhysicsComponent";
import ScriptComponent from "./ScriptComponent";
import SpawnPointComponent from "./SpawnPointComponent";
import TransformComponent from "./TransformComponent";

const COMPONENT_TYPE = {
  Mesh: "Mesh",
  Physics: "Physics",
  SpawnPoint: "Spawn Point",
  Script: "Script",
} as const;

type ComponentType = (typeof COMPONENT_TYPE)[keyof typeof COMPONENT_TYPE];

export default function InspectMenu() {
  const selectedId = useEditorStore((state) => state.selectedId);
  const node = useNode(selectedId);
  const name = useNodeAttribute(selectedId, "name");
  const meshId = useNodeAttribute(selectedId, "mesh");
  const extensions = useNodeAttribute(selectedId, "extensions");
  const extras = useNodeAttribute(selectedId, "extras");
  const spawn = useSpawn();

  const [open, setOpen] = useState(false);

  if (!node || !selectedId) return null;

  const availableComponents: ComponentType[] = [COMPONENT_TYPE.Script];

  if (!meshId) availableComponents.push(COMPONENT_TYPE.Mesh);
  if (!extensions?.OMI_collider) availableComponents.push(COMPONENT_TYPE.Physics);
  if (!extensions?.OMI_spawn_point && !spawn) availableComponents.push(COMPONENT_TYPE.SpawnPoint);

  return (
    <div className="pr-2 pb-4">
      <div className="flex w-full items-center justify-center pt-4">
        <input
          type="text"
          value={name ?? ""}
          onChange={(e) => {
            node.setName(e.target.value);
          }}
          className="mx-10 w-full rounded-lg py-0.5 text-center text-2xl font-bold transition hover:bg-neutral-200/80 focus:bg-neutral-200/80"
        />
      </div>

      <div className="space-y-4 px-1">
        <TransformComponent nodeId={selectedId} />

        {meshId && <MeshComponent meshId={meshId} />}
        {extensions?.OMI_collider && <PhysicsComponent nodeId={selectedId} />}
        {extensions?.OMI_spawn_point && <SpawnPointComponent nodeId={selectedId} />}

        {extras?.scripts?.map(({ id }) => {
          return <ScriptComponent key={id} nodeId={selectedId} scriptId={id} />;
        })}

        {availableComponents.length > 0 && (
          <div className="flex w-full justify-center">
            <DropdownMenu open={open} onOpenChange={setOpen}>
              <DropdownTrigger asChild>
                <Button className="rounded-lg px-8">Add Component</Button>
              </DropdownTrigger>

              <DropdownContent open={open}>
                <div className="py-2">
                  {availableComponents.includes("Mesh") && (
                    <ComponentButton
                      onClick={() => {
                        const { engine } = useEditorStore.getState();
                        if (!engine) return;

                        const { object: mesh } = engine.scene.mesh.create({
                          extras: {
                            customMesh: {
                              type: "Box",
                              width: 1,
                              height: 1,
                              depth: 1,
                            },
                          },
                        });

                        node.setMesh(mesh);
                      }}
                    >
                      {COMPONENT_TYPE.Mesh}
                    </ComponentButton>
                  )}

                  {availableComponents.includes(COMPONENT_TYPE.Physics) && (
                    <ComponentButton
                      onClick={() => {
                        const { engine } = useEditorStore.getState();
                        if (!engine) return;

                        const collider = engine.scene.extensions.collider.createCollider();
                        collider.type = "trimesh";

                        node.setExtension(ColliderExtension.EXTENSION_NAME, collider);
                      }}
                    >
                      {COMPONENT_TYPE.Physics}
                    </ComponentButton>
                  )}

                  {availableComponents.includes(COMPONENT_TYPE.Script) && (
                    <ComponentButton
                      onClick={() => {
                        const { engine } = useEditorStore.getState();
                        if (!engine || !extras) return;

                        const newExtras = { ...extras };
                        if (!newExtras.scripts) newExtras.scripts = [];

                        newExtras.scripts.push({ id: nanoid(), name: "New Script" });

                        node.setExtras(newExtras);
                      }}
                    >
                      {COMPONENT_TYPE.Script}
                    </ComponentButton>
                  )}

                  {availableComponents.includes(COMPONENT_TYPE.SpawnPoint) && (
                    <ComponentButton
                      onClick={() => {
                        const { engine } = useEditorStore.getState();
                        if (!engine) return;

                        const spawnPoint = engine.scene.extensions.spawn.createSpawnPoint();
                        spawnPoint.title = SPAWN_TITLES.Default;
                        node.setExtension("OMI_spawn_point", spawnPoint);
                      }}
                    >
                      {COMPONENT_TYPE.SpawnPoint}
                    </ComponentButton>
                  )}
                </div>
              </DropdownContent>
            </DropdownMenu>
          </div>
        )}
      </div>
    </div>
  );
}

function ComponentButton({ children, ...props }: DropdownMenuItemProps) {
  return (
    <DropdownItem
      className="w-full cursor-default px-10 text-left outline-none hover:bg-neutral-200 focus:bg-neutral-200 active:opacity-80"
      {...props}
    >
      {children}
    </DropdownItem>
  );
}
