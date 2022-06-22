import { useModel } from "../../../hooks/useModel";
import { GLTF } from "./GLTF";

interface ModelProps {
  modelId: string;
}

export function Model({ modelId }: ModelProps) {
  const model = useModel(modelId);

  if (!model) return null;

  return <GLTF url={model} />;
}
