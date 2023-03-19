import { Node } from "@gltf-transform/core";
import { NodeExtras } from "engine";
import { ColliderExtension, SPAWN_TITLE } from "gltf-extensions";
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
import { useEditorStore } from "../../store";

export const COMPONENT_TYPE = {
  Mesh: "Mesh",
  Physics: "Physics",
  SpawnPoint: "Spawn Point",
  Script: "Script",
} as const;

export type ComponentType = (typeof COMPONENT_TYPE)[keyof typeof COMPONENT_TYPE];

interface Props {
  availableComponents: ComponentType[];
  node: Node;
  extras?: NodeExtras;
}

export default function AddComponentButton({ availableComponents, node, extras }: Props) {
  const isPlaying = useEditorStore((state) => state.isPlaying);

  const [open, setOpen] = useState(false);

  return (
    <div className="flex w-full justify-center">
      <DropdownMenu
        open={open}
        onOpenChange={(value) => {
          if (value && isPlaying) return;
          setOpen(value);
        }}
      >
        <DropdownTrigger asChild>
          <Button disabled={isPlaying} className="rounded-lg px-8">
            Add Component
          </Button>
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

                  mesh.setName(node.getName());
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
                  collider.setType("trimesh");

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
                  spawnPoint.setTitle(SPAWN_TITLE.Default);
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
  );
}

function ComponentButton({ children, ...props }: DropdownMenuItemProps) {
  return (
    <DropdownItem
      className="w-full cursor-default px-10 text-left outline-none focus:bg-neutral-200 active:opacity-80"
      {...props}
    >
      {children}
    </DropdownItem>
  );
}
