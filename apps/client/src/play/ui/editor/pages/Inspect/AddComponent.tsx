import {
  EditNode_Collider_Type,
  EditNode_RigidBody_Type,
} from "@unavi/protocol";

import { editNode } from "@/src/play/actions/editNode";
import {
  DropdownContent,
  DropdownItem,
  DropdownMenu,
  DropdownTrigger,
} from "@/src/ui/DropdownMenu";

import { useTreeValue } from "../../hooks/useTreeValue";

enum AddOption {
  Physics = "physics",
}

interface Props {
  entityId: bigint;
}

export default function AddComponent({ entityId }: Props) {
  const options: AddOption[] = [];

  const id = useTreeValue(entityId, "id");
  const name = useTreeValue(entityId, "name");
  const locked = useTreeValue(entityId, "locked");
  const rigidBodyType = useTreeValue(entityId, "rigidBodyType");
  const colliderType = useTreeValue(entityId, "colliderType");

  if (!rigidBodyType || !colliderType) {
    options.push(AddOption.Physics);
  }

  if (!id || !name || locked || options.length === 0) {
    return null;
  }

  function handleAdd(option: AddOption) {
    if (!name || !id) {
      return;
    }

    switch (option) {
      case AddOption.Physics: {
        editNode(id, {
          collider: {
            size: [1, 1, 1],
            type: EditNode_Collider_Type.BOX,
          },
          rigidBody: {
            type: EditNode_RigidBody_Type.STATIC,
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
