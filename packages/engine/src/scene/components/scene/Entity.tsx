import { IEntity } from "../../types";
import { EntityComponent } from "./EntityComponent";

interface Props {
  entity: IEntity;
}

export function Entity({ entity }: Props) {
  return (
    <EntityComponent entity={entity}>
      {entity.children.map((child) => (
        <Entity key={child.id} entity={child} />
      ))}
    </EntityComponent>
  );
}
