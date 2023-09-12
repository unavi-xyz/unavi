import Rotation from "./Rotation";
import Scale from "./Scale";
import Translation from "./Translation";

interface Props {
  entityId: bigint;
}

export default function Transform({ entityId }: Props) {
  return (
    <div className="space-y-1">
      <Translation entityId={entityId} />
      <Rotation entityId={entityId} />
      <Scale entityId={entityId} />
    </div>
  );
}
