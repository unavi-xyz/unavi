import { Canvas } from "@react-three/fiber";
import { ContactShadows, OrbitControls } from "@react-three/drei";
import { Vector3 } from "three";
import { Avatar } from "3d";

export default function ProfileAvatar() {
  return (
    <div className="w-full h-full">
      <Canvas className="rounded-3xl" camera={{ position: [0, 1.2, 1.6] }}>
        <OrbitControls
          makeDefault
          enablePan={false}
          target={new Vector3(0, 0.9, 0)}
          enableDamping
          dampingFactor={0.05}
          maxPolarAngle={Math.PI / 1.9}
        />

        <Avatar src="/models/Latifa.vrm" />

        <ContactShadows width={2} height={2} blur={8} />
        <directionalLight />
      </Canvas>
    </div>
  );
}
