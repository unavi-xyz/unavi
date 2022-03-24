import { FC, useContext, useEffect, useRef } from "react";
import { HeightfieldArgs, Triplet, useHeightfield } from "@react-three/cannon";
import { BufferGeometry, Float32BufferAttribute } from "three";

import { CoreProperties, Heightmap, Properties } from "../types";
import { SceneContext } from "../SceneContext";

export type IHeightmap = CoreProperties & Pick<Properties, "heightmap">;

export const heightmapDefaultProperties: IHeightmap = {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  heightmap: { data: [], width: 1 },
};

export function Heightmap({ properties = heightmapDefaultProperties }) {
  const { width, data } = properties.heightmap;

  const elementSize = width / data.length;
  const args: HeightfieldArgs = [data, { elementSize }];

  const rotation: Triplet = [
    -Math.PI / 2 + properties.rotation[0],
    properties.rotation[1],
    properties.rotation[2],
  ];
  const position: Triplet = [
    -width / 2 + properties.position[0],
    properties.position[1],
    width / 2 + properties.position[2],
  ];

  const [ref] = useHeightfield(() => ({
    args,
    position,
    rotation,
  }));

  const { debug } = useContext(SceneContext);

  return (
    <mesh ref={ref}>
      {debug ? (
        <meshStandardMaterial flatShading />
      ) : (
        <meshBasicMaterial visible={false} />
      )}
      <HeightmapGeometry heights={data} elementSize={elementSize} />
    </mesh>
  );
}

const HeightmapGeometry: FC<{
  elementSize: number;
  heights: number[][];
}> = ({ elementSize, heights }) => {
  const ref = useRef<BufferGeometry>(null);

  useEffect(() => {
    if (!ref.current) return;
    const dx = elementSize;
    const dy = elementSize;

    /* Create the vertex data from the heights. */
    const vertices = heights.flatMap((row, i) =>
      row.flatMap((z, j) => [i * dx, j * dy, z])
    );

    /* Create the faces. */
    const indices = [];
    for (let i = 0; i < heights.length - 1; i++) {
      for (let j = 0; j < heights[i].length - 1; j++) {
        const stride = heights[i].length;
        const index = i * stride + j;
        indices.push(index + 1, index + stride, index + stride + 1);
        indices.push(index + stride, index + 1, index);
      }
    }

    ref.current.setIndex(indices);
    ref.current.setAttribute(
      "position",
      new Float32BufferAttribute(vertices, 3)
    );
    ref.current.computeVertexNormals();
    ref.current.computeBoundingBox();
    ref.current.computeBoundingSphere();
  }, [heights]);

  return <bufferGeometry ref={ref} />;
};
