import { useEffect } from "react";
import { useSphere } from "@react-three/cannon";

import { Transform } from "../../../types";

interface Props {
  args: [number];
  transform: Transform;
}

export default function SphereCollider({ args, transform }: Props) {
  const [ref, api] = useSphere(() => ({
    args: args,
    type: "Static",
  }));

  useEffect(() => {
    api.position.set(...transform.position);
    api.rotation.set(...transform.rotation);
  }, [transform]);

  return <object3D ref={ref} />;
}
