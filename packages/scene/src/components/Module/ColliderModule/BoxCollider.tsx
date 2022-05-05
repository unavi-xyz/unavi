import { useEffect } from "react";
import { Triplet, useBox } from "@react-three/cannon";

import { Transform } from "../../../types";

interface Props {
  args: Triplet;
  transform: Transform;
}

export default function BoxCollider({ args, transform }: Props) {
  const [ref, api] = useBox(() => ({
    args: args,
    type: "Static",
  }));

  useEffect(() => {
    api.position.set(...transform.position);
    api.rotation.set(...transform.rotation);
  }, [transform]);

  return <object3D ref={ref} />;
}
