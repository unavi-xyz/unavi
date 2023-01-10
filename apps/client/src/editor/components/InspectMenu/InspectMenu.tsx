import { useNode } from "../../hooks/useNode";
import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { useEditorStore } from "../../store";
import TransformComponent from "./TransformComponent";

// enum ComponentType {
//   Mesh = "Mesh",
//   Physics = "Physics",
// }

export default function InspectMenu() {
  const selectedId = useEditorStore((state) => state.selectedId);
  const name = useNodeAttribute(selectedId, "name");
  const node = useNode(selectedId);

  // const [open, setOpen] = useState(false);

  if (!selectedId) return null;

  // const otherComponents = Object.values(ComponentType).filter((type) => {
  //   if (mesh && type === ComponentType.Mesh) return false;
  //   if (collider && type === ComponentType.Physics) return false;
  //   return true;
  // });

  return (
    <div className="pr-2">
      <div className="flex w-full items-center justify-center pt-4">
        <input
          type="text"
          value={name ?? ""}
          onChange={(e) => {
            node?.setName(e.target.value);
          }}
          className="mx-10 w-full rounded-lg py-0.5 text-center text-2xl font-bold transition hover:bg-neutral-100 hover:shadow-inner"
        />
      </div>

      <div className="space-y-4 px-1">
        <TransformComponent nodeId={selectedId} />

        {/* {mesh && <MeshComponent nodeId={selectedId} mesh={mesh} />}
        {collider && <PhysicsComponent nodeId={selectedId} />} */}

        {/* {otherComponents.length > 0 && (
          <div className="space-y-1 px-5">
            <Button fullWidth rounded="large" onClick={() => setOpen(true)}>
              Add Component
            </Button>

            <DropdownMenu open={open} onClose={() => setOpen(false)} fullWidth>
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
        )} */}
      </div>
    </div>
  );
}

// function ComponentButton({ children, ...props }: any) {
//   return (
//     <button className="w-full cursor-default rounded-lg transition hover:bg-sky-100" {...props}>
//       {children}
//     </button>
//   );
// }
