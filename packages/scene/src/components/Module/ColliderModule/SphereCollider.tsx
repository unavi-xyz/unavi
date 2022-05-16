import { useSphere } from "@react-three/cannon";
import { useEffect } from "react";

import { Transform } from "../../../types";

interface Props {
  radius: number;
  transform: Transform;
}

export default function SphereCollider({ radius, transform }: Props) {
  const largestScale = transform.scale.reduce((acc, value) =>
    Math.max(acc, value)
  );
  const args: [number] = [radius * largestScale];

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
