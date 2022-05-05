import { useEffect } from "react";
import { Triplet, useBox } from "@react-three/cannon";

import { Transform } from "../../../types";

interface Props {
  args: Triplet;
  transform: Transform;
}

export default function BoxCollider({ args: size, transform }: Props) {
  const args: Triplet = [
    size[0] * transform.scale[0],
    size[1] * transform.scale[1],
    size[2] * transform.scale[2],
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
