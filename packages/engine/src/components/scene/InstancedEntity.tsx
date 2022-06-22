import { Entity } from "../../types";
import { EntityComponent } from "./EntityComponent";

interface Props {
  entity: Entity;
}

export function InstancedEntity({ entity }: Props) {
  return (
    <EntityComponent entity={entity}>
      {entity.children.map((child) => (
        <InstancedEntity key={child.id} entity={child} />
      ))}
    </EntityComponent>
  );
}
