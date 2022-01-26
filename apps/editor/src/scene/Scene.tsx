import { useEffect, useState } from "react";
import { useThree } from "@react-three/fiber";
import { OrbitControls, Sky, TransformControls } from "@react-three/drei";
import { ASSETS, PROPERTIES, Ground } from "3d";

import { TOOLS, useStore } from "../state/useStore";

import Objects from "./Objects";

export default function Scene() {
  const selected = useStore((state) => state.selected);
  const selectedRef = useStore((state) => state.selectedRef);
  const tool = useStore((state) => state.tool);
  const setUsingGizmo = useStore((state) => state.setUsingGizmo);

  const [enabled, setEnabled] = useState(false);

  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(10, 10, 10);
  }, [camera]);

  function handleMouseDown() {
    setUsingGizmo(true);
  }

  function handleMouseUp() {
    setUsingGizmo(false);
  }

  useEffect(() => {
    if (!selected) {
      setEnabled(false);
      return;
    }

    const properties = ASSETS[selected.type].properties;

    const hasType =
      tool === TOOLS.translate
        ? properties.includes(PROPERTIES.position)
        : tool === TOOLS.rotate
        ? properties.includes(PROPERTIES.rotation)
        : tool === TOOLS.scale
        ? properties.includes(PROPERTIES.scale)
        : false;

    setEnabled(hasType);
  }, [selected, tool]);

  return (
    <group>
      <OrbitControls
        addEventListener={undefined}
        hasEventListener={undefined}
        removeEventListener={undefined}
        dispatchEvent={undefined}
        makeDefault
      />

      <ambientLight />
      <directionalLight />

      <Sky />
      <Ground />

      <TransformControls // I LOVE TYPESCRIPT
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
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        object={selectedRef}
        enabled={enabled}
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
