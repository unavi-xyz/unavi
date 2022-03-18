import { Triplet, useBox } from "@react-three/cannon";
import { Material } from "../Material";
import { Texture } from "../types";

export const boxDefaultParams = {
  position: [0, 0, 0] as Triplet,
  rotation: [0, 0, 0] as Triplet,
  scale: [1, 1, 1] as Triplet,
};

interface Props {
  params: typeof boxDefaultParams;
  textures: { [key: string]: Texture };
}

export function Box({ params, textures }: Props) {
  const args: Triplet = params.scale;

  const [ref] = useBox(() => ({
    args,
    position: params.position,
    rotation: params.rotation,
    type: "Static",
  }));

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={args} />
      <Material params={params} textures={textures} />
    </mesh>
  );
}
