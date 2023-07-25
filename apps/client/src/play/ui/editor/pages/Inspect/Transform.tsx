import Rotation from "./Rotation";
import Scale from "./Scale";
import Translation from "./Translation";

interface Props {
  id: bigint;
}

export default function Transform({ id }: Props) {
  return (
    <div className="space-y-1">
      <Translation id={id} />
      <Rotation id={id} />
      <Scale id={id} />
    </div>
  );
}
