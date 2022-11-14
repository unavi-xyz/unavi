import { BoxMesh } from "@wired-labs/engine";
import { useState } from "react";

import Button from "../../../ui/Button";
import DropdownMenu from "../../../ui/DropdownMenu";
import { addMesh } from "../../actions/AddMeshAction";
import { updateNode } from "../../actions/UpdateNodeAction";
import { useMesh } from "../../hooks/useMesh";
import { useNode } from "../../hooks/useNode";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import MeshComponent from "./mesh/MeshComponent";
import PhysicsComponent from "./PhysicsComponent";
import SceneComponent from "./SceneComponent";
import TransformComponent from "./TransformComponent";

enum ComponentType {
  Mesh = "Mesh",
  Physics = "Physics",
}

export default function InspectMenu() {
  const spawnId = useEditorStore((state) => state.engine?.scene.spawnId);
  const selectedId = useEditorStore((state) => state.selectedId);
  const node = useNode(selectedId);

  const name = useSubscribeValue(node?.name$);
  const meshId = useSubscribeValue(node?.meshId$);
  const collider = useSubscribeValue(node?.collider$);

  const mesh = useMesh(meshId);

  const [open, setOpen] = useState(false);

  if (!selectedId)
    return (
      <div className="pr-2">
        <div className="mt-0.5 w-full pt-4 text-center text-2xl font-bold">
          Scene
        </div>

        <SceneComponent />
      </div>
    );

  const isSpawn = selectedId === spawnId;

  const otherComponents = Object.values(ComponentType).filter((type) => {
    if (isSpawn) return false;
    if (mesh && type === ComponentType.Mesh) return false;
    if (collider && type === ComponentType.Physics) return false;
    return true;
  });

  return (
    <div className="pr-2">
      <div className="flex w-full items-center justify-center pt-4">
        <input
          type="text"
          value={name ?? ""}
          onChange={(e) => {
            updateNode(selectedId, { name: e.target.value });
          }}
          className="rounded-lg w-full mx-10 py-0.5 text-2xl text-center font-bold transition hover:bg-neutral-100 hover:shadow-inner"
        />
      </div>

      <div className="space-y-4">
        <TransformComponent nodeId={selectedId} />

        {mesh && <MeshComponent nodeId={selectedId} mesh={mesh} />}
        {collider && <PhysicsComponent nodeId={selectedId} />}

        {otherComponents.length > 0 && (
          <div className="px-5">
            <Button fullWidth rounded="large" onClick={() => setOpen(true)}>
              Add Component
            </Button>

            <DropdownMenu open={open} onClose={() => setOpen(false)}>
              <div className="space-y-1 p-2">
                {otherComponents.map((type) => {
                  switch (type) {
                    case ComponentType.Mesh: {
                      return (
                        <ComponentButton
                          key={type}
                          onClick={() => {
                            const mesh = new BoxMesh();
                            addMesh(mesh);
                            updateNode(selectedId, { meshId: mesh.id });
                          }}
                        >
                          Mesh
                        </ComponentButton>
                      );
                    }

                    case ComponentType.Physics: {
                      return (
                        <ComponentButton
                          key={type}
                          onClick={() => {
                            updateNode(selectedId, {
                              collider: {
                                type: "box",
                                size: [1, 1, 1],
                              },
                            });
                          }}
                        >
                          Physics
                        </ComponentButton>
                      );
                    }
                  }
                })}
              </div>
            </DropdownMenu>
          </div>
        )}
      </div>
    </div>
  );
}

function ComponentButton({ children, ...props }: any) {
  return (
    <button
      className="w-full cursor-default rounded-lg transition hover:bg-primaryContainer"
      {...props}
    >
      {children}
    </button>
  );
}
