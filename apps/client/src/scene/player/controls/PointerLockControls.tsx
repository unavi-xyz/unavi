import React, { useEffect } from "react";
import { useThree } from "@react-three/fiber";
import { Camera, Euler, EventDispatcher, Vector3 } from "three";

//modified from @react-three/drei to add the "paused" prop
export const PointerLockControls = React.forwardRef(
  ({ selector, onChange, onLock, onUnlock, ...props }: any, ref) => {
    const { camera, paused, ...rest } = props;

    const gl = useThree(({ gl }) => gl);
    const defaultCamera = useThree(({ camera }) => camera);
    const invalidate = useThree(({ invalidate }) => invalidate);
    const explCamera = camera || defaultCamera;

    const [controls] = React.useState(
      () => new PointerLockControlsImpl(explCamera, gl.domElement)
    );

    useEffect(() => {
      if (paused) {
        controls?.dispatchEvent({ type: "pause" });
      } else {
        controls?.dispatchEvent({ type: "unpause" });
      }
    }, [paused]);

    useEffect(() => {
      const callback = (e) => {
        invalidate();
        if (onChange) onChange(e);
      };

      controls?.addEventListener?.("change", callback);

      if (onLock) controls?.addEventListener?.("lock", onLock);
      if (onUnlock) controls?.addEventListener?.("unlock", onUnlock);

      return () => {
        controls?.removeEventListener?.("change", callback);
        if (onLock) controls?.addEventListener?.("lock", onLock);
        if (onUnlock) controls?.addEventListener?.("unlock", onUnlock);
      };
    }, [onChange, onLock, onUnlock, controls, invalidate]);

    useEffect(() => {
      const handler = () => controls?.lock();
      const elements = selector
        ? Array.from(document.querySelectorAll(selector))
        : [document];
      elements.forEach(
        (element) => element && element.addEventListener("click", handler)
      );
      return () => {
        elements.forEach((element) =>
          element ? element.removeEventListener("click", handler) : undefined
        );
      };
    }, [controls, selector]);

    return controls ? (
      <primitive ref={ref} dispose={undefined} object={controls} {...rest} />
    ) : null;
  }
);

class PointerLockControlsImpl extends EventDispatcher {
  camera;
  domElement;

  isLocked = false;
  isPaused = false;

  // Set to constrain the pitch of the camera
  // Range is 0 to Math.PI radians
  minPolarAngle = 0; // radians
  maxPolarAngle = Math.PI; // radians

  changeEvent = { type: "change" };
  lockEvent = { type: "lock" };
  unlockEvent = { type: "unlock" };

  euler = new Euler(0, 0, 0, "YXZ");

  PI_2 = Math.PI / 2;

  vec = new Vector3();

  constructor(camera: Camera, domElement: HTMLCanvasElement) {
    super();

    this.domElement = domElement;
    this.camera = camera;

    this.isPaused = true;

    this.connect();
  }

  onMouseMove = (event) => {
    if (this.isLocked === false || this.isPaused) return;

    const movementX =
      event.movementX || event.mozMovementX || event.webkitMovementX || 0;
    const movementY =
      event.movementY || event.mozMovementY || event.webkitMovementY || 0;

    this.euler.setFromQuaternion(this.camera.quaternion);

    this.euler.y -= movementX * 0.002;
    this.euler.x -= movementY * 0.002;

    this.euler.x = Math.max(
      this.PI_2 - this.maxPolarAngle,
      Math.min(this.PI_2 - this.minPolarAngle, this.euler.x)
    );

    this.camera.quaternion.setFromEuler(this.euler);

    this.dispatchEvent(this.changeEvent);
  };

  onPointerlockChange = () => {
    if (this.domElement.ownerDocument.pointerLockElement === this.domElement) {
      this.dispatchEvent(this.lockEvent);

      this.isLocked = true;
    } else {
      this.dispatchEvent(this.unlockEvent);

      this.isLocked = false;
    }
  };

  onPointerlockError = () => {
    console.error("THREE.PointerLockControls: Unable to use Pointer Lock API");
  };

  onPause = () => {
    this.isPaused = true;
  };

  onUnPause = () => {
    this.isPaused = false;
  };

  connect = () => {
    this.domElement.ownerDocument.addEventListener(
      "mousemove",
      this.onMouseMove
    );
    this.domElement.ownerDocument.addEventListener(
      "pointerlockchange",
      this.onPointerlockChange
    );
    this.domElement.ownerDocument.addEventListener(
      "pointerlockerror",
      this.onPointerlockError
    );
    this.addEventListener("pause", this.onPause);
    this.addEventListener("unpause", this.onUnPause);
  };

  disconnect = () => {
    this.domElement.ownerDocument.removeEventListener(
      "mousemove",
      this.onMouseMove
    );
    this.domElement.ownerDocument.removeEventListener(
      "pointerlockchange",
      this.onPointerlockChange
    );
    this.domElement.ownerDocument.removeEventListener(
      "pointerlockerror",
      this.onPointerlockError
    );
    this.removeEventListener("pause", this.onPause);
    this.removeEventListener("unpause", this.onUnPause);
  };

  dispose = () => {
    this.disconnect();
  };

  getObject = () =>
    // retaining this method for backward compatibility
    this.camera;

  direction = new Vector3(0, 0, -1);
  getDirection = (v: Vector3) =>
    v.copy(this.direction).applyQuaternion(this.camera.quaternion);

  moveForward = (distance: number) => {
    // move forward parallel to the xz-plane
    // assumes this.camera.up is y-up

    this.vec.setFromMatrixColumn(this.camera.matrix, 0);

    this.vec.crossVectors(this.camera.up, this.vec);

    this.camera.position.addScaledVector(this.vec, distance);
  };

  moveRight = (distance: number) => {
    this.vec.setFromMatrixColumn(this.camera.matrix, 0);

    this.camera.position.addScaledVector(this.vec, distance);
  };

  lock = () => {
    this.domElement.requestPointerLock();
  };

  unlock = () => {
    this.domElement.ownerDocument.exitPointerLock();
  };
}
