import {
  editNode,
  SyncedNode,
  SyncedNode_Collider_Type,
  SyncedNode_RigidBody_Type,
} from "@unavi/engine";

import { DeepReadonly } from "@/src/play/utils/types";
import {
  DropdownContent,
  DropdownItem,
  DropdownMenu,
  DropdownTrigger,
} from "@/src/ui/DropdownMenu";

enum AddOption {
  Physics = "physics",
}

interface Props {
  node: DeepReadonly<SyncedNode>;
}

export default function AddComponent({ node }: Props) {
  const options: AddOption[] = [];

  if (node?.collider.type === SyncedNode_Collider_Type.NONE) {
    options.push(AddOption.Physics);
  }

  if (!node || options.length === 0) {
    return null;
  }

  function handleAdd(option: AddOption) {
    switch (option) {
      case AddOption.Physics: {
        editNode(node.id, {
          collider: {
            height: 1,
            meshId: "",
            radius: 0.5,
            size: [1, 1, 1],
            type: SyncedNode_Collider_Type.BOX,
          },
          rigidBody: {
            type: SyncedNode_RigidBody_Type.STATIC,
          },
        });
        break;
      }
    }
  }

  return (
    <DropdownMenu>
      <DropdownTrigger asChild>
        <button className="w-full rounded-lg py-1 text-center transition hover:bg-neutral-800">
          Add Component
        </button>
      </DropdownTrigger>

      <DropdownContent className="z-50 w-full animate-scaleIn rounded-lg bg-neutral-800 shadow-dark">
        {options.map((option) => (
          <DropdownItem
            key={option}
            onClick={() => handleAdd(option)}
            className="w-full cursor-pointer rounded-lg px-20 py-1 text-center capitalize text-white transition hover:bg-neutral-700"
          >
            {option}
          </DropdownItem>
        ))}
      </DropdownContent>
    </DropdownMenu>
  );
}
