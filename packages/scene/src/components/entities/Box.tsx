import { Triplet, useBox } from "@react-three/cannon";
import { useEffect, useRef, useState } from "react";
import { Mesh } from "three";

import { useGlobalTransform } from "../../hooks/useGlobalTransform";
import { useMaterial } from "../../hooks/useMaterial";
import { Transform } from "../../types";
import { Material } from "./Material";

export interface BoxProps {
  width?: number;
  height?: number;
  depth?: number;
  physics?: boolean;
  materialId?: string;
}

export function Box({
  width = 1,
  height = 1,
  depth = 1,
  physics = true,
  materialId,
}: BoxProps) {
  const ref = useRef<Mesh>(null);
  const [key, setKey] = useState(0);

  const material = useMaterial(materialId);
  const globalTransform = useGlobalTransform(ref);

  useEffect(() => {
    setKey((prev) => prev + 1);
  }, [width, height, depth, globalTransform]);

  return (
    <mesh ref={ref}>
      <boxBufferGeometry args={[width, height, depth]} />
      <Material {...material} />

      {physics && (
        <BoxCollider
          key={key}
          width={width}
          height={height}
          depth={depth}
          transform={globalTransform}
        />
      )}
    </mesh>
  );
}

export interface BoxColliderProps {
  width: number;
  height: number;
  depth: number;
  transform: Transform;
}

function BoxCollider({ width, height, depth, transform }: BoxColliderProps) {
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
