import { useEffect, useState } from "react";
import { Raycaster } from "three";
import { TransformControls } from "three/examples/jsm/controls/TransformControls";

import { useStudioStore } from "../store";
import { getObject } from "../utils/scene";

export function useTransformControls() {
  const engine = useStudioStore((state) => state.engine);
  const selectedId = useStudioStore((state) => state.selectedId);
  const tool = useStudioStore((state) => state.tool);

  const [controls, setControls] = useState<TransformControls>();

  // Create transform controls
  useEffect(() => {
    if (!engine) return;
    const { camera, renderer, scene } = engine.renderManager;

    const transformControls = new TransformControls(camera, renderer.domElement);
    setControls(transformControls);
    scene.add(transformControls);

    const raycaster = new Raycaster();

    function onTransformMouseDown() {
      useStudioStore.setState({ usingTransformControls: true });
    }

    function onCanvasMouseUp() {
      useStudioStore.setState({ usingTransformControls: false });
    }

    function onCanvasMouseDown(event: MouseEvent) {
      const { usingTransformControls, root } = useStudioStore.getState();
      if (usingTransformControls) return;

      // Get mouse position on the canvas
      const box = renderer.domElement.getBoundingClientRect();
      const x = event.clientX - box.left;
      const y = event.clientY - box.top;

      const mouseX = (x / renderer.domElement.scrollWidth) * 2 - 1;
      const mouseY = -(y / renderer.domElement.scrollHeight) * 2 + 1;

      // Set raycaster
      raycaster.setFromCamera(
        {
          x: mouseX,
          y: mouseY,
        },
        camera
      );

      // Get intersected objects
      const intersected = raycaster.intersectObject(root);
      const object = intersected.length > 0 ? intersected[0].object : null;

      if (object) {
        useStudioStore.setState({ selectedId: object.uuid });
        return;
      }

      useStudioStore.setState({ selectedId: null });
    }

    transformControls.addEventListener("mouseDown", onTransformMouseDown);
    renderer.domElement.addEventListener("mouseup", onCanvasMouseUp);
    renderer.domElement.addEventListener("mousedown", onCanvasMouseDown);

    return () => {
      transformControls.removeEventListener("mouseDown", onTransformMouseDown);
      renderer.domElement.removeEventListener("mouseup", onCanvasMouseUp);
      renderer.domElement.removeEventListener("mousedown", onCanvasMouseDown);
      scene.remove(transformControls);
    };
  }, [engine]);

  // Attach controls to selected object
  useEffect(() => {
    if (!engine || !controls) return;

    const object = selectedId ? getObject(selectedId) : undefined;

    if (!object) {
      controls.detach();
      return;
    }

    controls.attach(object);
  }, [engine, controls, selectedId]);

  // Switch controls to tool
  useEffect(() => {
    if (!engine || !controls) return;
    controls.setMode(tool);
  }, [engine, controls, tool]);
}
