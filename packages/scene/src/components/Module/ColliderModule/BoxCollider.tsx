import { Triplet, useBox } from "@react-three/cannon";
import { useEffect } from "react";

import { Transform } from "../../../types";

interface Props {
  width: number;
  height: number;
  depth: number;
  transform: Transform;
}

export default function BoxCollider({
  width,
  height,
  depth,
  transform,
}: Props) {
  const args: Triplet = [
    width * transform.scale[0],
    height * transform.scale[1],
    depth * transform.scale[2],
  ];

  const [ref, api] = useBox(() => ({
    args,
    type: "Static",
    position: transform.position,
    rotation: transform.rotation,
  }));

  useEffect(() => {
    api.position.set(...transform.position);
    api.rotation.set(...transform.rotation);
  }, [transform]);

  return <object3D ref={ref} />;
}
