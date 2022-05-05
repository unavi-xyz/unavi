import { useEffect } from "react";
import { useSphere } from "@react-three/cannon";

import { Transform } from "../../../types";

interface Props {
  args: [number];
  transform: Transform;
}

export default function SphereCollider({ args: radius, transform }: Props) {
  const largestScale = transform.scale.reduce((acc, value) =>
    Math.max(acc, value)
  );
  const args: [number] = [radius[0] * largestScale];

  const [ref, api] = useSphere(() => ({
    args: args,
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
