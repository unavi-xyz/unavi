import {
  EditNode_Collider_Type,
  EditNode_RigidBody_Type,
} from "@unavi/protocol";

import { editNode } from "@/src/play/actions/editNode";

import { useTreeValue } from "../../hooks/useTreeValue";
import { useTreeValueKey } from "../../hooks/useTreeValueKey";
import InspectSection from "./InspectSection";
import NumberInput from "./NumberInput";
import { SelectInput } from "./SelectInput";

interface Props {
  id: bigint;
}

export default function Physics({ id }: Props) {
  const name = useTreeValue(id, "name");
  const locked = useTreeValue(id, "locked");
  const colliderType = useTreeValue(id, "colliderType");
  const rigidBodyType = useTreeValue(id, "rigidBodyType");

  const size = useTreeValueKey(id, "collider", "size") ?? [0, 0, 0];
  const height = useTreeValueKey(id, "collider", "height") ?? 0;
  const radius = useTreeValueKey(id, "collider", "radius") ?? 0;

  if (!name || !colliderType || !rigidBodyType) {
    return null;
  }

  const handleRemove = locked
    ? undefined
    : () => {
      editNode({
        collider: {
          type: EditNode_Collider_Type.NONE,
        },
        rigidBody: {
          type: EditNode_RigidBody_Type.NONE,
        },
        target: name,
      });
    };

  const rigidBodyOption = getRigidBodyOption(rigidBodyType);
  const colliderOption = getColliderOption(colliderType);

  return (
    <InspectSection title="Physics" onRemove={handleRemove}>
      <SelectInput
        label="Type"
        disabled={locked}
        value={rigidBodyOption}
        options={[RigidBodyOption.Static, RigidBodyOption.Dynamic]}
        onChange={(e) => {
          const value = e.currentTarget.value as RigidBodyOption;
          const rigidBodyType = getRigidBodyType(value);
          editNode({
            rigidBody: {
              type: rigidBodyType,
            },
            target: name,
          });
        }}
      />

      <SelectInput
        label="Shape"
        disabled={locked}
        value={colliderOption}
        options={[
          ColliderOption.Box,
          ColliderOption.Sphere,
          ColliderOption.Cylinder,
          ColliderOption.Capsule,
          ColliderOption.Mesh,
          ColliderOption.Hull,
        ]}
        onChange={(e) => {
          const value = e.currentTarget.value as ColliderOption;
          const colliderType = getColliderType(value);
          editNode({
            collider: {
              height: 1,
              radius: 0.5,
              size: [1, 1, 1],
              type: colliderType,
            },
            target: name,
          });
        }}
      />

      {colliderType === EditNode_Collider_Type.BOX && (
        <div className="grid grid-cols-4">
          <div className="w-20 shrink-0 font-bold text-neutral-400">Size</div>

          <NumberInput
            label="X"
            disabled={locked}
            min={0}
            value={size[0]}
            onValueChange={(val) => {
              editNode({
                collider: {
                  size: [val, size[1], size[2]],
                  type: colliderType,
                },
                target: name,
              });
            }}
          />
          <NumberInput
            label="Y"
            disabled={locked}
            min={0}
            value={size[1]}
            onValueChange={(val) => {
              editNode({
                collider: {
                  size: [size[0], val, size[2]],
                  type: colliderType,
                },
                target: name,
              });
            }}
          />
          <NumberInput
            label="Z"
            disabled={locked}
            min={0}
            value={size[2]}
            onValueChange={(val) => {
              editNode({
                collider: {
                  size: [size[0], size[1], val],
                  type: colliderType,
                },
                target: name,
              });
            }}
          />
        </div>
      )}

      {colliderType === EditNode_Collider_Type.SPHERE ||
        colliderType === EditNode_Collider_Type.CAPSULE ||
        colliderType === EditNode_Collider_Type.CYLINDER ? (
        <div className="grid grid-cols-4">
          <div className="w-20 shrink-0 font-bold text-neutral-400">Radius</div>

          <div className="col-span-3">
            <NumberInput
              disabled={locked}
              min={0}
              value={radius}
              onValueChange={(val) => {
                editNode({
                  collider: {
                    height,
                    radius: val,
                    type: colliderType,
                  },
                  target: name,
                });
              }}
            />
          </div>
        </div>
      ) : null}

      {colliderType === EditNode_Collider_Type.CAPSULE ||
        colliderType === EditNode_Collider_Type.CYLINDER ? (
        <div className="grid grid-cols-4">
          <div className="w-20 shrink-0 font-bold text-neutral-400">Height</div>

          <div className="col-span-3">
            <NumberInput
              disabled={locked}
              min={0}
              value={height}
              onValueChange={(val) => {
                editNode({
                  collider: {
                    height: val,
                    radius,
                    type: colliderType,
                  },
                  target: name,
                });
              }}
            />
          </div>
        </div>
      ) : null}
    </InspectSection>
  );
}

enum RigidBodyOption {
  Static = "static",
  Dynamic = "dynamic",
}

function getRigidBodyOption(type: EditNode_RigidBody_Type): RigidBodyOption {
  switch (type) {
    case EditNode_RigidBody_Type.STATIC: {
      return RigidBodyOption.Static;
    }

    case EditNode_RigidBody_Type.DYNAMIC: {
      return RigidBodyOption.Dynamic;
    }

    default: {
      return RigidBodyOption.Static;
    }
  }
}

function getRigidBodyType(option: RigidBodyOption): EditNode_RigidBody_Type {
  switch (option) {
    case RigidBodyOption.Static: {
      return EditNode_RigidBody_Type.STATIC;
    }

    case RigidBodyOption.Dynamic: {
      return EditNode_RigidBody_Type.DYNAMIC;
    }
  }
}

enum ColliderOption {
  Box = "box",
  Sphere = "sphere",
  Cylinder = "cylinder",
  Capsule = "capsule",
  Mesh = "mesh",
  Hull = "hull",
}

function getColliderOption(type: EditNode_Collider_Type): ColliderOption {
  switch (type) {
    case EditNode_Collider_Type.BOX: {
      return ColliderOption.Box;
    }

    case EditNode_Collider_Type.SPHERE: {
      return ColliderOption.Sphere;
    }

    case EditNode_Collider_Type.CYLINDER: {
      return ColliderOption.Cylinder;
    }

    case EditNode_Collider_Type.CAPSULE: {
      return ColliderOption.Capsule;
    }

    case EditNode_Collider_Type.MESH: {
      return ColliderOption.Mesh;
    }

    case EditNode_Collider_Type.HULL: {
      return ColliderOption.Hull;
    }

    default: {
      return ColliderOption.Box;
    }
  }
}

function getColliderType(option: ColliderOption): EditNode_Collider_Type {
  switch (option) {
    case ColliderOption.Box: {
      return EditNode_Collider_Type.BOX;
    }

    case ColliderOption.Sphere: {
      return EditNode_Collider_Type.SPHERE;
    }

    case ColliderOption.Cylinder: {
      return EditNode_Collider_Type.CYLINDER;
    }

    case ColliderOption.Capsule: {
      return EditNode_Collider_Type.CAPSULE;
    }

    case ColliderOption.Mesh: {
      return EditNode_Collider_Type.MESH;
    }

    case ColliderOption.Hull: {
      return EditNode_Collider_Type.HULL;
    }
  }
}
