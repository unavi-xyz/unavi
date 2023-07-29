import { ColliderType, RigidBodyType } from "@unavi/protocol";

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

  return (
    <InspectSection title="Physics">
      <SelectInput
        label="Type"
        disabled={locked}
        value={rigidBodyType}
        options={[RigidBodyType.Static, RigidBodyType.Dynamic]}
        onChange={(e) => {
          const value = e.currentTarget.value as RigidBodyType;
          editNode({
            rigidBodyType: value,
            target: name,
          });
        }}
      />

      <SelectInput
        label="Shape"
        disabled={locked}
        value={colliderType}
        options={[
          ColliderType.Box,
          ColliderType.Sphere,
          ColliderType.Cylinder,
          ColliderType.Capsule,
          ColliderType.Mesh,
          ColliderType.Hull,
        ]}
        onChange={(e) => {
          const value = e.currentTarget.value as ColliderType;
          editNode({
            collider: {
              height: 1,
              radius: 0.5,
              size: [1, 1, 1],
              type: value,
            },
            target: name,
          });
        }}
      />

      {colliderType === ColliderType.Box && (
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

      {colliderType === ColliderType.Sphere ||
        colliderType === ColliderType.Cylinder ||
        colliderType === ColliderType.Capsule ? (
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

      {colliderType === ColliderType.Cylinder ||
        colliderType === ColliderType.Capsule ? (
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
