import { Vector3 } from "three";

const distanceX = 0.8;
const distanceY = 0.7;

const positions = [
  new Vector3(0, 0, -100),
  new Vector3(distanceX, 0, -100),
  new Vector3(-distanceX, 0, -100),
  new Vector3(0, distanceY, -100),
  new Vector3(0, -distanceY, -100),
];

export default function Crosshair() {
  return (
    <group>
      {positions.map((position, i) => {
        return <Circle key={i} position={position} />;
      })}
    </group>
  );
}

function Circle({ position }: { position: Vector3 }) {
  return (
    <mesh position={position} renderOrder={999}>
      <sphereGeometry args={[0.11, 8, 8]} />
      <meshBasicMaterial
        color="white"
        depthTest={false}
        depthWrite={false}
        transparent={true}
      />
    </mesh>
  );
}
