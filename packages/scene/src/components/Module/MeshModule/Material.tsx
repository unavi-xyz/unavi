import { useMaterial } from "../../MaterialProvider";

interface Props {
  materialId: string | undefined;
}

export default function Material({ materialId }: Props) {
  const material = useMaterial(materialId);

  return <meshStandardMaterial color={material?.color ?? "#ffffff"} />;
}
