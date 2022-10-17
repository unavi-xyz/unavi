import { BoxMesh } from "@wired-labs/engine";
import { useState } from "react";

import Button from "../../../ui/Button";
import DropdownMenu from "../../../ui/DropdownMenu";
import { updateEntity } from "../../actions/UpdateEntityAction";
import { useEntity } from "../../hooks/useEntity";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import MeshComponent from "./mesh/MeshComponent";
import PhysicsComponent from "./PhysicsComponent";
import TransformComponent from "./TransformComponent";

enum ComponentType {
  Mesh = "Mesh",
  Physics = "Physics",
}

export default function InspectMenu() {
  const selectedId = useEditorStore((state) => state.selectedId);
  const entity = useEntity(selectedId);

  const name = useSubscribeValue(entity?.name$);
  const mesh = useSubscribeValue(entity?.mesh$);
  const collider = useSubscribeValue(entity?.collider$);

  const [open, setOpen] = useState(false);

  if (!selectedId) return null;

  const otherComponents = Object.values(ComponentType).filter((type) => {
    if (mesh && type === ComponentType.Mesh) return false;
    if (collider && type === ComponentType.Physics) return false;
    return true;
  });

  return (
    <div className="pr-2" key={selectedId}>
      <div className="flex w-full items-center justify-center pt-4">
        <input
          type="text"
          value={name ?? ""}
          onChange={(e) => {
            updateEntity(selectedId, { name: e.target.value });
          }}
          className="rounded-lg py-0.5 text-center text-2xl font-bold transition hover:bg-neutral-100 hover:shadow-inner"
        />
      </div>

      <div className="space-y-4">
        <TransformComponent entityId={selectedId} />

        {mesh && <MeshComponent entityId={selectedId} mesh={mesh} />}
        {collider && <PhysicsComponent entityId={selectedId} />}

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
                            updateEntity(selectedId, {
                              mesh: mesh.toJSON(),
                            });
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
                            updateEntity(selectedId, {
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
