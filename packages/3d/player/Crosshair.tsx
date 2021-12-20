const distanceX = 1;
const distanceY = 1;

const positions = [
  [0, 0],
  [distanceX, 0],
  [-distanceX, 0],
  [0, distanceY],
  [0, -distanceY],
];

export default function Crosshair() {
  return (
    <group>
      {positions.map((pos, i) => {
        return <Circle key={i} position={pos} />;
      })}
    </group>
  );
}

function Circle({ position }) {
  return (
    <mesh position={[...position, -100]} renderOrder={999}>
      <sphereGeometry args={[0.1, 8, 8]} />
      <meshBasicMaterial
        color="white"
        depthTest={false}
        depthWrite={false}
        transparent={true}
      />
    </mesh>
  );
}
