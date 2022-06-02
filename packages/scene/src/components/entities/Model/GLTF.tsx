import { useGLTF } from "@react-three/drei";
import { Suspense } from "react";

interface GLTFProps {
  url: string;
}

export function GLTF({ url }: GLTFProps) {
  const gltf = useGLTF(url);

  return (
    <Suspense fallback={null}>
      <primitive object={gltf.scene.clone()} />
    </Suspense>
  );
}
