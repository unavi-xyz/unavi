import { useEffect, useRef, useState } from "react";
import { Raycaster, Vector2 } from "three";
import { TransformControls } from "three/examples/jsm/controls/TransformControls";

import { useStudioStore } from "../store";
import { UserData } from "../types";
import { findObject } from "../utils/scene";

export function useTransformControls() {
  const engine = useStudioStore((state) => state.engine);
  const selectedId = useStudioStore((state) => state.selectedId);
  const tool = useStudioStore((state) => state.tool);

  const mouseRef = useRef(new Vector2());
  const raycasterRef = useRef(new Raycaster());

  const [controls, setControls] = useState<TransformControls>();

  // Create transform controls
  useEffect(() => {
    if (!engine) return;

    const canvas = engine.renderer.domElement;
    const transformControls = new TransformControls(engine.camera, canvas);
    engine.scene.add(transformControls);

    function onMouseDown(event: MouseEvent) {
      if (!engine || !canvas) return;

      // If currently using transform controls, don't select anything
      if (useStudioStore.getState().usingTransform) return;

      event.preventDefault();

      // Get mouse position on the canvas
      const box = canvas.getBoundingClientRect();
      const x = event.clientX - box.left;
      const y = event.clientY - box.top;

      mouseRef.current.x = (x / canvas.scrollWidth) * 2 - 1;
      mouseRef.current.y = -(y / canvas.scrollHeight) * 2 + 1;

      // Set raycaster
      raycasterRef.current.setFromCamera(mouseRef.current, engine.camera);

      // Get intersected objects
      const intersected = raycasterRef.current.intersectObjects(engine.scene.children);

      // Find the first object that is part of the tree
      const intersection = intersected.find(({ object }) => {
        const userData: UserData = object.userData;
        return userData.treeNode;
      });

      // If no valid object was found, remove selectedId
      if (!intersection) {
        useStudioStore.setState({ selectedId: null });
        return;
      }

      useStudioStore.setState({ selectedId: intersection.object.uuid });
    }

    function onTransformDown() {
      useStudioStore.setState({ usingTransform: true });
    }

    function onMouseUp() {
      useStudioStore.setState({ usingTransform: false });
    }

    canvas.addEventListener("mousedown", onMouseDown);
    canvas.addEventListener("mouseup", onMouseUp);
    transformControls.addEventListener("mouseDown", onTransformDown);

    setControls(transformControls);

    return () => {
      canvas.removeEventListener("mousedown", onMouseDown);
      canvas.removeEventListener("mouseup", onMouseUp);
      transformControls.removeEventListener("mouseDown", onTransformDown);
      transformControls.removeFromParent();
      transformControls.dispose();
    };
  }, [engine]);

  // Attach controls to selected object
  useEffect(() => {
    if (!engine || !controls || !selectedId) return;

    const selectedObject = findObject(selectedId);
    if (!selectedObject) return;

    controls.attach(selectedObject);

    return () => {
      controls.detach();
    };
  }, [engine, controls, selectedId]);

  // Switch controls to tool
  useEffect(() => {
    if (!engine || !controls) return;
    controls.setMode(tool);
  }, [engine, controls, tool]);
}
