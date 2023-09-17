import {
  editNode,
  SyncedNode,
  SyncedNode_Collider_Type,
  SyncedNode_RigidBody_Type,
} from "@unavi/engine";

import { DeepReadonly } from "@/src/play/utils/types";

import InspectSection from "./InspectSection";
import NumberInput from "./NumberInput";
import { SelectInput } from "./SelectInput";

interface Props {
  node: DeepReadonly<SyncedNode>;
}

export default function Physics({ node }: Props) {
  const handleRemove = node.locked
    ? undefined
    : () => {
        editNode(node.id, {
          collider: {
            height: 1,
            meshId: "",
            radius: 1,
            size: [1, 1, 1],
            type: SyncedNode_Collider_Type.NONE,
          },
        });
      };

  const sizeX = node.collider.size[0] ?? 0;
  const sizeY = node.collider.size[1] ?? 0;
  const sizeZ = node.collider.size[2] ?? 0;

  return (
    <InspectSection title="Physics" onRemove={handleRemove}>
      <SelectInput
        label="Type"
        disabled={node.locked}
        value={node.rigidBody.type}
        options={[
          SyncedNode_RigidBody_Type.STATIC,
          SyncedNode_RigidBody_Type.DYNAMIC,
        ]}
        onChange={(e) => {
          const value = e.currentTarget.value as SyncedNode_RigidBody_Type;

          editNode(node.id, {
            rigidBody: {
              type: value,
            },
          });
        }}
      />

      <SelectInput
        label="Shape"
        disabled={node.locked}
        value={node.collider.type}
        options={[
          SyncedNode_Collider_Type.BOX,
          SyncedNode_Collider_Type.SPHERE,
          SyncedNode_Collider_Type.CAPSULE,
          SyncedNode_Collider_Type.CYLINDER,
          SyncedNode_Collider_Type.MESH,
          SyncedNode_Collider_Type.HULL,
        ]}
        onChange={(e) => {
          const value = e.currentTarget.value as SyncedNode_Collider_Type;

          editNode(node.id, {
            collider: {
              ...node.collider,
              size: [sizeX, sizeY, sizeZ],
              type: value,
            },
          });
        }}
      />

      {node.collider.type === SyncedNode_Collider_Type.BOX && (
        <div className="grid grid-cols-4">
          <div className="w-20 shrink-0 font-bold text-neutral-400">Size</div>

          <NumberInput
            label="X"
            disabled={node.locked}
            min={0}
            value={round(sizeX)}
            onValueChange={(val) => {
              editNode(node.id, {
                collider: {
                  ...node.collider,
                  size: [val, sizeY, sizeZ],
                },
              });
            }}
          />
          <NumberInput
            label="Y"
            disabled={node.locked}
            min={0}
            value={round(sizeY)}
            onValueChange={(val) => {
              editNode(node.id, {
                collider: {
                  ...node.collider,
                  size: [sizeX, val, sizeZ],
                },
              });
            }}
          />
          <NumberInput
            label="Z"
            disabled={node.locked}
            min={0}
            value={round(sizeZ)}
            onValueChange={(val) => {
              editNode(node.id, {
                collider: {
                  ...node.collider,
                  size: [sizeX, sizeY, val],
                },
              });
            }}
          />
        </div>
      )}

      {node.collider.type === SyncedNode_Collider_Type.SPHERE ||
      node.collider.type === SyncedNode_Collider_Type.CAPSULE ||
      node.collider.type === SyncedNode_Collider_Type.CYLINDER ? (
        <div className="grid grid-cols-4">
          <div className="w-20 shrink-0 font-bold text-neutral-400">Radius</div>

          <div className="col-span-3">
            <NumberInput
              disabled={node.locked}
              min={0}
              value={round(node.collider.radius)}
              onValueChange={(val) => {
                editNode(node.id, {
                  collider: {
                    ...node.collider,
                    radius: val,
                    size: [sizeX, sizeY, sizeZ],
                  },
                });
              }}
            />
          </div>
        </div>
      ) : null}

      {node.collider.type === SyncedNode_Collider_Type.CAPSULE ||
      node.collider.type === SyncedNode_Collider_Type.CYLINDER ? (
        <div className="grid grid-cols-4">
          <div className="w-20 shrink-0 font-bold text-neutral-400">Height</div>

          <div className="col-span-3">
            <NumberInput
              disabled={node.locked}
              min={0}
              value={round(node.collider.height)}
              onValueChange={(val) => {
                editNode(node.id, {
                  collider: {
                    ...node.collider,
                    height: val,
                    size: [sizeX, sizeY, sizeZ],
                  },
                });
              }}
            />
          </div>
        </div>
      ) : null}
    </InspectSection>
  );
}

function round(num: number, precision = 1000) {
  return Math.round(num * precision) / precision;
}
