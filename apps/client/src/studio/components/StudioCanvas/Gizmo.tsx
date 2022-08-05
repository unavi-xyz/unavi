import { Triplet } from "@react-three/cannon";
import { TransformControls } from "@react-three/drei";
import { useAtomValue } from "jotai";

import { selectedAtom, selectedRefAtom } from "../../../studio/atoms";
import { useStudioStore } from "../../../studio/store";

export default function Gizmo() {
  const selected = useAtomValue(selectedAtom);
  const selectedRef = useAtomValue(selectedRefAtom);
  const tool = useStudioStore((state) => state.tool);

  const visible = Boolean(selected) && Boolean(selectedRef);

  function handleMouseUp() {
    useStudioStore.setState({ usingTransform: false });

    if (!selected || !selectedRef?.current) return;

    //get the transform
    const position = selectedRef.current.position.toArray();
    const scale = selectedRef.current.scale.toArray();
    const rotationArray = selectedRef.current.rotation.toArray();
    const rotation: Triplet = [rotationArray[0], rotationArray[1], rotationArray[2]];

    const transform = { position, rotation, scale };

    //update the object state
    useStudioStore.getState().updateEntity(selected?.id, (draft) => {
      draft.transform = transform;
    });
  }

  return (
    <TransformControls
      object={selectedRef?.current ?? undefined}
      mode={tool}
      showX={visible}
      showY={visible}
      showZ={visible}
      enabled={visible}
      onMouseDown={() => useStudioStore.setState({ usingTransform: true })}
      onMouseUp={handleMouseUp}
    />
  );
}
