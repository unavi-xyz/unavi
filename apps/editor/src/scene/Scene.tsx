import { useEffect } from "react";
import { useThree } from "@react-three/fiber";
import { OrbitControls, TransformControls } from "@react-three/drei";

import { useStore } from "../state";
import Objects from "./Objects";

const GRID_SIZE = 20;

export default function Scene() {
  const selectedRef = useStore((state) => state.selectedRef);
  const tool = useStore((state) => state.tool);

  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(9, 9, 9);
  }, [camera]);

  return (
    <group>
      <OrbitControls
        addEventListener={undefined}
        hasEventListener={undefined}
        removeEventListener={undefined}
        dispatchEvent={undefined}
        makeDefault
      />
      <gridHelper args={[GRID_SIZE, GRID_SIZE]} />
      <ambientLight />
      <directionalLight />

      <TransformControls // kek
        type={undefined}
        isGroup={undefined}
        id={undefined}
        uuid={undefined}
        name={undefined}
        parent={undefined}
        modelViewMatrix={undefined}
        normalMatrix={undefined}
        matrixWorld={undefined}
        matrixAutoUpdate={undefined}
        matrixWorldNeedsUpdate={undefined}
        castShadow={undefined}
        receiveShadow={undefined}
        frustumCulled={undefined}
        renderOrder={undefined}
        animations={undefined}
        userData={undefined}
        customDepthMaterial={undefined}
        customDistanceMaterial={undefined}
        isObject3D={undefined}
        onBeforeRender={undefined}
        onAfterRender={undefined}
        applyMatrix4={undefined}
        applyQuaternion={undefined}
        setRotationFromAxisAngle={undefined}
        setRotationFromEuler={undefined}
        setRotationFromMatrix={undefined}
        setRotationFromQuaternion={undefined}
        rotateOnAxis={undefined}
        rotateOnWorldAxis={undefined}
        rotateX={undefined}
        rotateY={undefined}
        rotateZ={undefined}
        translateOnAxis={undefined}
        translateX={undefined}
        translateY={undefined}
        translateZ={undefined}
        localToWorld={undefined}
        worldToLocal={undefined}
        lookAt={undefined}
        add={undefined}
        remove={undefined}
        removeFromParent={undefined}
        clear={undefined}
        getObjectById={undefined}
        getObjectByName={undefined}
        getObjectByProperty={undefined}
        getWorldPosition={undefined}
        getWorldQuaternion={undefined}
        getWorldScale={undefined}
        getWorldDirection={undefined}
        raycast={undefined}
        traverse={undefined}
        traverseVisible={undefined}
        traverseAncestors={undefined}
        updateMatrix={undefined}
        updateWorldMatrix={undefined}
        toJSON={undefined}
        clone={undefined}
        copy={undefined}
        addEventListener={undefined}
        hasEventListener={undefined}
        removeEventListener={undefined}
        dispatchEvent={undefined}
        object={selectedRef}
        enabled={Boolean(selectedRef)}
        showX={Boolean(selectedRef)}
        showY={Boolean(selectedRef)}
        showZ={Boolean(selectedRef)}
        size={0.7}
        mode={tool}
      />

      <Objects />
    </group>
  );
}
