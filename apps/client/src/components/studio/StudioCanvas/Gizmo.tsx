import { Triplet } from "@react-three/cannon";
import { TransformControls } from "@react-three/drei";

import { useStudioStore } from "../../../helpers/studio/store";

export default function Gizmo() {
  const selected = useStudioStore((state) => state.selected);
  const selectedRef = useStudioStore((state) =>
    selected?.id ? state.treeRefs[selected.id] : null
  );

  const visible = Boolean(selected) && Boolean(selectedRef);

  function handleMouseDown() {
    useStudioStore.setState({ usingGizmo: true });
  }

  function handleMouseUp() {
    useStudioStore.setState({ usingGizmo: false });

    if (!selected || !selectedRef?.current) return;

    //get the position and rotation
    const position = selectedRef.current.position.toArray();
    const rotationArray = selectedRef.current.rotation.toArray();
    const rotation: Triplet = [
      rotationArray[0],
      rotationArray[1],
      rotationArray[2],
    ];

    //update the object state
    useStudioStore
      .getState()
      .updateObject(selected?.id, { position, rotation });
  }

  return (
    <TransformControls
      object={selectedRef?.current}
      showX={visible}
      showY={visible}
      showZ={visible}
      enabled={visible}
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
    />
  );
}
