import { useTreeValue } from "../../hooks/useTreeValue";
import InspectSection from "./InspectSection";

interface Props {
  id: bigint;
}

export default function Physics({ id }: Props) {
  const name = useTreeValue(id, "name");
  const colliderType = useTreeValue(id, "colliderType");
  const rigidBodyType = useTreeValue(id, "rigidBodyType");

  if (!colliderType && !rigidBodyType) {
    return null;
  }

  return (
    <InspectSection title="Physics">
      <div></div>
    </InspectSection>
  );
}
