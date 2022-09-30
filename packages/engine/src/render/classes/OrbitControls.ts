// @ts-nocheck
import {
  EventDispatcher,
  Matrix4,
  MOUSE,
  OrthographicCamera,
  PerspectiveCamera,
  Quaternion,
  Spherical,
  TOUCH,
  Vector2,
  Vector3,
} from "three";

import { PointerData, WheelData } from "../types";

// https://github.com/mrdoob/three.js/blob/dev/examples/jsm/controls/OrbitControls.js

// This set of controls performs orbiting, dollying (zooming), and panning.
// Unlike TrackballControls, it maintains the "up" direction object.up (+Y by default).
//
//    Orbit - left mouse / touch: one-finger move
//    Zoom - middle mouse, or mousewheel / touch: two-finger spread or squish
//    Pan - right mouse, or left mouse + ctrl/meta/shiftKey, or arrow keys / touch: two-finger move

export type FakeKeyDownEvent = CustomEvent<{ code: string }>;
export type FakePointerEvent = CustomEvent<PointerData>;
export type FakeWheelEvent = CustomEvent<WheelData>;

const _changeEvent = { type: "change" };
const _startEvent = { type: "start" };
const _endEvent = { type: "end" };

export class OrbitControls extends EventDispatcher {
  object: PerspectiveCamera | OrthographicCamera;
  domElement: EventTarget;
  enabled: boolean;
  target: Vector3;
  minDistance: number;
  maxDistance: number;
  minZoom: number;
  maxZoom: number;
  minPolarAngle: number;
  maxPolarAngle: number;
  minAzimuthAngle: number;
  maxAzimuthAngle: number;
  enableDamping: boolean;
  dampingFactor: number;
  enableZoom: boolean;
  zoomSpeed: number;
  enableRotate: boolean;
  rotateSpeed: number;
  enablePan: boolean;
  panSpeed: number;
  screenSpacePanning: boolean;
  keyPanSpeed: number;
  autoRotate: boolean;
  autoRotateSpeed: number;
  target0: Vector3;
  position0: Vector3;
  zoom0: number;
  _domElementKeyEvents: EventTarget | null;

  keys = {
    LEFT: "ArrowLeft",
    UP: "ArrowUp",
    RIGHT: "ArrowRight",
    BOTTOM: "ArrowDown",
  };
  mouseButtons = { LEFT: MOUSE.ROTATE, MIDDLE: MOUSE.DOLLY, RIGHT: MOUSE.PAN };
  touches = { ONE: TOUCH.ROTATE, TWO: TOUCH.DOLLY_PAN };

  getPolarAngle: () => number;
  getAzimuthalAngle: () => number;
  getDistance: () => number;
  listenToKeyEvents: (domElement: EventTarget) => void;
  saveState: () => void;
  reset: () => void;
  update: () => void;
  dispose: () => void;

  constructor(
    object: PerspectiveCamera | OrthographicCamera,
    domElement: EventTarget,
    clientWidth: number,
    clientHeight: number
  ) {
    super();

    if (domElement === undefined)
      console.warn(
        'THREE.OrbitControls: The second parameter "domElement" is now mandatory.'
      );

    this.object = object;
    this.domElement = domElement;

    // Set to false to disable this control
    this.enabled = true;

    // "target" sets the location of focus, where the object orbits around
    this.target = new Vector3();

    // How far you can dolly in and out ( PerspectiveCamera only )
    this.minDistance = 0;
    this.maxDistance = Infinity;

    // How far you can zoom in and out ( OrthographicCamera only )
    this.minZoom = 0;
    this.maxZoom = Infinity;

    // How far you can orbit vertically, upper and lower limits.
    // Range is 0 to Math.PI radians.
    this.minPolarAngle = 0; // radians
    this.maxPolarAngle = Math.PI; // radians

    // How far you can orbit horizontally, upper and lower limits.
    // If set, the interval [ min, max ] must be a sub-interval of [ - 2 PI, 2 PI ], with ( max - min < 2 PI )
    this.minAzimuthAngle = -Infinity; // radians
    this.maxAzimuthAngle = Infinity; // radians

    // Set to true to enable damping (inertia)
    // If damping is enabled, you must call controls.update() in your animation loop
    this.enableDamping = false;
    this.dampingFactor = 0.05;

    // This option actually enables dollying in and out; left as "zoom" for backwards compatibility.
    // Set to false to disable zooming
    this.enableZoom = true;
    this.zoomSpeed = 1.0;

    // Set to false to disable rotating
    this.enableRotate = true;
    this.rotateSpeed = 1.0;

    // Set to false to disable panning
    this.enablePan = true;
    this.panSpeed = 1.0;
    this.screenSpacePanning = true; // if false, pan orthogonal to world-space direction camera.up
    this.keyPanSpeed = 7.0; // pixels moved per arrow key push

    // Set to true to automatically rotate around the target
    // If auto-rotate is enabled, you must call controls.update() in your animation loop
    this.autoRotate = false;
    this.autoRotateSpeed = 2.0; // 30 seconds per orbit when fps is 60

    // for reset
    this.target0 = this.target.clone();
    this.position0 = this.object.position.clone();
    this.zoom0 = this.object.zoom;

    // the target DOM element for key events
    this._domElementKeyEvents = null;

    //
    // public methods
    //

    this.getPolarAngle = function () {
      return spherical.phi;
    };

    this.getAzimuthalAngle = function () {
      return spherical.theta;
    };

    this.getDistance = function () {
      return this.object.position.distanceTo(this.target);
    };

    this.listenToKeyEvents = function (domElement) {
      //@ts-ignore
      domElement.addEventListener("keydown", onKeyDown);
      this._domElementKeyEvents = domElement;
    };

    this.saveState = function () {
      this.target0.copy(this.target);
      this.position0.copy(this.object.position);
      this.zoom0 = this.object.zoom;
    };

    this.reset = function () {
      this.target.copy(this.target0);
      this.object.position.copy(this.position0);
      this.object.zoom = this.zoom0;

      this.object.updateProjectionMatrix();
      this.dispatchEvent(_changeEvent);

      this.update();

      state = STATE.NONE;
    };

    // this method is exposed, but perhaps it would be better if we can make it private...
    this.update = (function () {
      const offset = new Vector3();

      // so camera.up is the orbit axis
      const quat = new Quaternion().setFromUnitVectors(
        object.up,
        new Vector3(0, 1, 0)
      );
      const quatInverse = quat.clone().invert();

      const lastPosition = new Vector3();
      const lastQuaternion = new Quaternion();

      const twoPI = 2 * Math.PI;

      return function update() {
        const position = this.object.position;

        offset.copy(position).sub(this.target);

        // rotate offset to "y-axis-is-up" space
        offset.applyQuaternion(quat);

        // angle from z-axis around y-axis
        spherical.setFromVector3(offset);

        if (this.autoRotate && state === STATE.NONE) {
          rotateLeft(getAutoRotationAngle());
        }

        if (this.enableDamping) {
          spherical.theta += sphericalDelta.theta * this.dampingFactor;
          spherical.phi += sphericalDelta.phi * this.dampingFactor;
        } else {
          spherical.theta += sphericalDelta.theta;
          spherical.phi += sphericalDelta.phi;
        }

        // restrict theta to be between desired limits

        let min = this.minAzimuthAngle;
        let max = this.maxAzimuthAngle;

        if (isFinite(min) && isFinite(max)) {
          if (min < -Math.PI) min += twoPI;
          else if (min > Math.PI) min -= twoPI;

          if (max < -Math.PI) max += twoPI;
          else if (max > Math.PI) max -= twoPI;

          if (min <= max) {
            spherical.theta = Math.max(min, Math.min(max, spherical.theta));
          } else {
            spherical.theta =
              spherical.theta > (min + max) / 2
                ? Math.max(min, spherical.theta)
                : Math.min(max, spherical.theta);
          }
        }

        // restrict phi to be between desired limits
        spherical.phi = Math.max(
          this.minPolarAngle,
          Math.min(this.maxPolarAngle, spherical.phi)
        );

        spherical.makeSafe();

        spherical.radius *= scale;

        // restrict radius to be between desired limits
        spherical.radius = Math.max(
          this.minDistance,
          Math.min(this.maxDistance, spherical.radius)
        );

        // move target to panned location

        if (this.enableDamping === true) {
          this.target.addScaledVector(panOffset, this.dampingFactor);
        } else {
          this.target.add(panOffset);
        }

        offset.setFromSpherical(spherical);

        // rotate offset back to "camera-up-vector-is-up" space
        offset.applyQuaternion(quatInverse);

        position.copy(this.target).add(offset);

        this.object.lookAt(this.target);

        if (this.enableDamping === true) {
          sphericalDelta.theta *= 1 - this.dampingFactor;
          sphericalDelta.phi *= 1 - this.dampingFactor;

          panOffset.multiplyScalar(1 - this.dampingFactor);
        } else {
          sphericalDelta.set(0, 0, 0);

          panOffset.set(0, 0, 0);
        }

        scale = 1;

        // update condition is:
        // min(camera displacement, camera rotation in radians)^2 > EPS
        // using small-angle approximation cos(x/2) = 1 - x^2 / 8

        if (
          zoomChanged ||
          lastPosition.distanceToSquared(this.object.position) > EPS ||
          8 * (1 - lastQuaternion.dot(this.object.quaternion)) > EPS
        ) {
          this.dispatchEvent(_changeEvent);

          lastPosition.copy(this.object.position);
          lastQuaternion.copy(this.object.quaternion);
          zoomChanged = false;

          return true;
        }

        return false;
      };
    })();

    this.dispose = function () {
      //@ts-ignore
      this.domElement.removeEventListener("pointerdown", onPointerDown);
      //@ts-ignore
      this.domElement.removeEventListener("pointercancel", onPointerCancel);
      //@ts-ignore
      this.domElement.removeEventListener("wheel", onMouseWheel);
      //@ts-ignore
      this.domElement.removeEventListener("pointermove", onPointerMove);
      //@ts-ignore
      this.domElement.removeEventListener("pointerup", onPointerUp);

      if (this._domElementKeyEvents !== null) {
        //@ts-ignore
        this._domElementKeyEvents.removeEventListener("keydown", onKeyDown);
      }

      //this.dispatchEvent( { type: 'dispose' } ); // should this be added here?
    };

    //
    // internals
    //

    const STATE = {
      NONE: -1,
      ROTATE: 0,
      DOLLY: 1,
      PAN: 2,
      TOUCH_ROTATE: 3,
      TOUCH_PAN: 4,
      TOUCH_DOLLY_PAN: 5,
      TOUCH_DOLLY_ROTATE: 6,
    };

    let state = STATE.NONE;

    const EPS = 0.000001;

    // current position in spherical coordinates
    const spherical = new Spherical();
    const sphericalDelta = new Spherical();

    let scale = 1;
    const panOffset = new Vector3();
    let zoomChanged = false;

    const rotateStart = new Vector2();
    const rotateEnd = new Vector2();
    const rotateDelta = new Vector2();

    const panStart = new Vector2();
    const panEnd = new Vector2();
    const panDelta = new Vector2();

    const dollyStart = new Vector2();
    const dollyEnd = new Vector2();
    const dollyDelta = new Vector2();

    const pointers: FakePointerEvent[] = [];
    const pointerPositions: {
      [pointerId: number]: Vector2;
    } = {};

    function getAutoRotationAngle() {
      return ((2 * Math.PI) / 60 / 60) * this.autoRotateSpeed;
    }

    function getZoomScale() {
      return Math.pow(0.95, this.zoomSpeed);
    }

    function rotateLeft(angle: number) {
      sphericalDelta.theta -= angle;
    }

    function rotateUp(angle: number) {
      sphericalDelta.phi -= angle;
    }

    const panLeft = (function () {
      const v = new Vector3();

      return function panLeft(distance: number, objectMatrix: Matrix4) {
        v.setFromMatrixColumn(objectMatrix, 0); // get X column of objectMatrix
        v.multiplyScalar(-distance);

        panOffset.add(v);
      };
    })();

    const panUp = (function () {
      const v = new Vector3();

      return function panUp(distance: number, objectMatrix: Matrix4) {
        if (this.screenSpacePanning === true) {
          v.setFromMatrixColumn(objectMatrix, 1);
        } else {
          v.setFromMatrixColumn(objectMatrix, 0);
          v.crossVectors(this.object.up, v);
        }

        v.multiplyScalar(distance);

        panOffset.add(v);
      };
    })();

    // deltaX and deltaY are in pixels; right and down are positive
    const pan = (function () {
      const offset = new Vector3();

      return function pan(deltaX: number, deltaY: number) {
        if (this.object instanceof PerspectiveCamera) {
          // perspective
          const position = this.object.position;
          offset.copy(position).sub(this.target);
          let targetDistance = offset.length();

          // half of the fov is center to top of screen
          targetDistance *= Math.tan(((this.object.fov / 2) * Math.PI) / 180.0);

          // we use only clientHeight here so aspect ratio does not distort speed
          panLeft(
            (2 * deltaX * targetDistance) / clientHeight,
            this.object.matrix
          );
          panUp(
            (2 * deltaY * targetDistance) / clientHeight,
            this.object.matrix
          );
        } else if (this.object instanceof OrthographicCamera) {
          // orthographic
          panLeft(
            (deltaX * (this.object.right - this.object.left)) /
              this.object.zoom /
              clientWidth,
            this.object.matrix
          );
          panUp(
            (deltaY * (this.object.top - this.object.bottom)) /
              this.object.zoom /
              clientHeight,
            this.object.matrix
          );
        } else {
          // camera neither orthographic nor perspective
          console.warn(
            "WARNING: OrbitControls.js encountered an unknown camera type - pan disabled."
          );
          this.enablePan = false;
        }
      };
    })();

    function dollyOut(dollyScale: number) {
      if (this.object instanceof PerspectiveCamera) {
        scale /= dollyScale;
      } else if (this.object instanceof OrthographicCamera) {
        this.object.zoom = Math.max(
          this.minZoom,
          Math.min(this.maxZoom, this.object.zoom * dollyScale)
        );
        this.object.updateProjectionMatrix();
        zoomChanged = true;
      } else {
        console.warn(
          "WARNING: OrbitControls.js encountered an unknown camera type - dolly/zoom disabled."
        );
        this.enableZoom = false;
      }
    }

    function dollyIn(dollyScale: number) {
      if (this.object instanceof PerspectiveCamera) {
        scale *= dollyScale;
      } else if (this.object instanceof OrthographicCamera) {
        this.object.zoom = Math.max(
          this.minZoom,
          Math.min(this.maxZoom, this.object.zoom / dollyScale)
        );
        this.object.updateProjectionMatrix();
        zoomChanged = true;
      } else {
        console.warn(
          "WARNING: OrbitControls.js encountered an unknown camera type - dolly/zoom disabled."
        );
        this.enableZoom = false;
      }
    }

    //
    // event callbacks - update the object state
    //

    function handleMouseDownRotate(event: FakePointerEvent) {
      rotateStart.set(event.detail.clientX, event.detail.clientY);
    }

    function handleMouseDownDolly(event: FakePointerEvent) {
      dollyStart.set(event.detail.clientX, event.detail.clientY);
    }

    function handleMouseDownPan(event: FakePointerEvent) {
      panStart.set(event.detail.clientX, event.detail.clientY);
    }

    function handleMouseMoveRotate(event: FakePointerEvent) {
      rotateEnd.set(event.detail.clientX, event.detail.clientY);

      rotateDelta
        .subVectors(rotateEnd, rotateStart)
        .multiplyScalar(this.rotateSpeed);

      rotateLeft((2 * Math.PI * rotateDelta.x) / clientHeight); // yes, height

      rotateUp((2 * Math.PI * rotateDelta.y) / clientHeight);

      rotateStart.copy(rotateEnd);

      this.update();
    }

    function handleMouseMoveDolly(event: FakePointerEvent) {
      dollyEnd.set(event.detail.clientX, event.detail.clientY);

      dollyDelta.subVectors(dollyEnd, dollyStart);

      if (dollyDelta.y > 0) {
        dollyOut(getZoomScale());
      } else if (dollyDelta.y < 0) {
        dollyIn(getZoomScale());
      }

      dollyStart.copy(dollyEnd);

      this.update();
    }

    function handleMouseMovePan(event: FakePointerEvent) {
      panEnd.set(event.detail.clientX, event.detail.clientY);

      panDelta.subVectors(panEnd, panStart).multiplyScalar(this.panSpeed);

      pan(panDelta.x, panDelta.y);

      panStart.copy(panEnd);

      this.update();
    }

    function handleMouseWheel(event: FakeWheelEvent) {
      if (event.detail.deltaY < 0) {
        dollyIn(getZoomScale());
      } else if (event.detail.deltaY > 0) {
        dollyOut(getZoomScale());
      }

      this.update();
    }

    function handleKeyDown(event: FakeKeyDownEvent) {
      let needsUpdate = false;
      switch (event.detail.code) {
        case this.keys.UP:
          pan(0, this.keyPanSpeed);
          needsUpdate = true;
          break;

        case this.keys.BOTTOM:
          pan(0, -this.keyPanSpeed);
          needsUpdate = true;
          break;

        case this.keys.LEFT:
          pan(this.keyPanSpeed, 0);
          needsUpdate = true;
          break;

        case this.keys.RIGHT:
          pan(-this.keyPanSpeed, 0);
          needsUpdate = true;
          break;
      }

      if (needsUpdate) this.update();
    }

    function handleTouchStartRotate() {
      if (pointers.length === 1) {
        rotateStart.set(pointers[0].detail.pageX, pointers[0].detail.pageY);
      } else {
        const x = 0.5 * (pointers[0].detail.pageX + pointers[1].detail.pageX);
        const y = 0.5 * (pointers[0].detail.pageY + pointers[1].detail.pageY);

        rotateStart.set(x, y);
      }
    }

    function handleTouchStartPan() {
      if (pointers.length === 1) {
        panStart.set(pointers[0].detail.pageX, pointers[0].detail.pageY);
      } else {
        const x = 0.5 * (pointers[0].detail.pageX + pointers[1].detail.pageX);
        const y = 0.5 * (pointers[0].detail.pageY + pointers[1].detail.pageY);

        panStart.set(x, y);
      }
    }

    function handleTouchStartDolly() {
      const dx = pointers[0].detail.pageX - pointers[1].detail.pageX;
      const dy = pointers[0].detail.pageY - pointers[1].detail.pageY;

      const distance = Math.sqrt(dx * dx + dy * dy);

      dollyStart.set(0, distance);
    }

    function handleTouchStartDollyPan() {
      if (this.enableZoom) handleTouchStartDolly();

      if (this.enablePan) handleTouchStartPan();
    }

    function handleTouchStartDollyRotate() {
      if (this.enableZoom) handleTouchStartDolly();

      if (this.enableRotate) handleTouchStartRotate();
    }

    function handleTouchMoveRotate(event: FakePointerEvent) {
      if (pointers.length == 1) {
        rotateEnd.set(event.detail.pageX, event.detail.pageY);
      } else {
        const position = getSecondPointerPosition(event);

        const x = 0.5 * (event.detail.pageX + position.x);
        const y = 0.5 * (event.detail.pageY + position.y);

        rotateEnd.set(x, y);
      }

      rotateDelta
        .subVectors(rotateEnd, rotateStart)
        .multiplyScalar(this.rotateSpeed);

      rotateLeft((2 * Math.PI * rotateDelta.x) / clientHeight); // yes, height

      rotateUp((2 * Math.PI * rotateDelta.y) / clientHeight);

      rotateStart.copy(rotateEnd);
    }

    function handleTouchMovePan(event: FakePointerEvent) {
      if (pointers.length === 1) {
        panEnd.set(event.detail.pageX, event.detail.pageY);
      } else {
        const position = getSecondPointerPosition(event);

        const x = 0.5 * (event.detail.pageX + position.x);
        const y = 0.5 * (event.detail.pageY + position.y);

        panEnd.set(x, y);
      }

      panDelta.subVectors(panEnd, panStart).multiplyScalar(this.panSpeed);

      pan(panDelta.x, panDelta.y);

      panStart.copy(panEnd);
    }

    function handleTouchMoveDolly(event: FakePointerEvent) {
      const position = getSecondPointerPosition(event);

      const dx = event.detail.pageX - position.x;
      const dy = event.detail.pageY - position.y;

      const distance = Math.sqrt(dx * dx + dy * dy);

      dollyEnd.set(0, distance);

      dollyDelta.set(0, Math.pow(dollyEnd.y / dollyStart.y, this.zoomSpeed));

      dollyOut(dollyDelta.y);

      dollyStart.copy(dollyEnd);
    }

    function handleTouchMoveDollyPan(event: FakePointerEvent) {
      if (this.enableZoom) handleTouchMoveDolly(event);

      if (this.enablePan) handleTouchMovePan(event);
    }

    function handleTouchMoveDollyRotate(event: FakePointerEvent) {
      if (this.enableZoom) handleTouchMoveDolly(event);

      if (this.enableRotate) handleTouchMoveRotate(event);
    }

    //
    // event handlers - FSM: listen for events and reset state
    //

    function onPointerDown(event: FakePointerEvent) {
      if (this.enabled === false) return;

      if (pointers.length === 0) {
        //@ts-ignore
        this.domElement.addEventListener("pointermove", onPointerMove);
        //@ts-ignore
        this.domElement.addEventListener("pointerup", onPointerUp);
      }

      //

      addPointer(event);

      if (event.detail.pointerType === "touch") {
        onTouchStart(event);
      } else {
        onMouseDown(event);
      }
    }

    function onPointerMove(event: FakePointerEvent) {
      if (this.enabled === false) return;

      if (event.detail.pointerType === "touch") {
        onTouchMove(event);
      } else {
        onMouseMove(event);
      }
    }

    function onPointerUp(event: FakePointerEvent) {
      removePointer(event);

      if (pointers.length === 0) {
        //@ts-ignore
        this.domElement.removeEventListener("pointermove", onPointerMove);
        //@ts-ignore
        this.domElement.removeEventListener("pointerup", onPointerUp);
      }

      this.dispatchEvent(_endEvent);

      state = STATE.NONE;
    }

    function onPointerCancel(event: FakePointerEvent) {
      removePointer(event);
    }

    function onMouseDown(event: FakePointerEvent) {
      let mouseAction;

      switch (event.detail.button) {
        case 0:
          mouseAction = this.mouseButtons.LEFT;
          break;

        case 1:
          mouseAction = this.mouseButtons.MIDDLE;
          break;

        case 2:
          mouseAction = this.mouseButtons.RIGHT;
          break;

        default:
          mouseAction = -1;
      }

      switch (mouseAction) {
        case MOUSE.DOLLY:
          if (this.enableZoom === false) return;

          handleMouseDownDolly(event);

          state = STATE.DOLLY;

          break;

        case MOUSE.ROTATE:
          if (
            event.detail.ctrlKey ||
            event.detail.metaKey ||
            event.detail.shiftKey
          ) {
            if (this.enablePan === false) return;

            handleMouseDownPan(event);

            state = STATE.PAN;
          } else {
            if (this.enableRotate === false) return;

            handleMouseDownRotate(event);

            state = STATE.ROTATE;
          }

          break;

        case MOUSE.PAN:
          if (
            event.detail.ctrlKey ||
            event.detail.metaKey ||
            event.detail.shiftKey
          ) {
            if (this.enableRotate === false) return;

            handleMouseDownRotate(event);

            state = STATE.ROTATE;
          } else {
            if (this.enablePan === false) return;

            handleMouseDownPan(event);

            state = STATE.PAN;
          }

          break;

        default:
          state = STATE.NONE;
      }

      if (state !== STATE.NONE) {
        this.dispatchEvent(_startEvent);
      }
    }

    function onMouseMove(event: FakePointerEvent) {
      if (this.enabled === false) return;

      switch (state) {
        case STATE.ROTATE:
          if (this.enableRotate === false) return;

          handleMouseMoveRotate(event);

          break;

        case STATE.DOLLY:
          if (this.enableZoom === false) return;

          handleMouseMoveDolly(event);

          break;

        case STATE.PAN:
          if (this.enablePan === false) return;

          handleMouseMovePan(event);

          break;
      }
    }

    function onMouseWheel(event: FakeWheelEvent) {
      if (
        this.enabled === false ||
        this.enableZoom === false ||
        state !== STATE.NONE
      )
        return;

      this.dispatchEvent(_startEvent);

      handleMouseWheel(event);

      this.dispatchEvent(_endEvent);
    }

    function onKeyDown(event: FakeKeyDownEvent) {
      if (this.enabled === false || this.enablePan === false) return;

      handleKeyDown(event);
    }

    function onTouchStart(event: FakePointerEvent) {
      trackPointer(event);

      switch (pointers.length) {
        case 1:
          switch (this.touches.ONE) {
            case TOUCH.ROTATE:
              if (this.enableRotate === false) return;

              handleTouchStartRotate();

              state = STATE.TOUCH_ROTATE;

              break;

            case TOUCH.PAN:
              if (this.enablePan === false) return;

              handleTouchStartPan();

              state = STATE.TOUCH_PAN;

              break;

            default:
              state = STATE.NONE;
          }

          break;

        case 2:
          switch (this.touches.TWO) {
            case TOUCH.DOLLY_PAN:
              if (this.enableZoom === false && this.enablePan === false) return;

              handleTouchStartDollyPan();

              state = STATE.TOUCH_DOLLY_PAN;

              break;

            case TOUCH.DOLLY_ROTATE:
              if (this.enableZoom === false && this.enableRotate === false)
                return;

              handleTouchStartDollyRotate();

              state = STATE.TOUCH_DOLLY_ROTATE;

              break;

            default:
              state = STATE.NONE;
          }

          break;

        default:
          state = STATE.NONE;
      }

      if (state !== STATE.NONE) {
        this.dispatchEvent(_startEvent);
      }
    }

    function onTouchMove(event: FakePointerEvent) {
      trackPointer(event);

      switch (state) {
        case STATE.TOUCH_ROTATE:
          if (this.enableRotate === false) return;

          handleTouchMoveRotate(event);

          this.update();

          break;

        case STATE.TOUCH_PAN:
          if (this.enablePan === false) return;

          handleTouchMovePan(event);

          this.update();

          break;

        case STATE.TOUCH_DOLLY_PAN:
          if (this.enableZoom === false && this.enablePan === false) return;

          handleTouchMoveDollyPan(event);

          this.update();

          break;

        case STATE.TOUCH_DOLLY_ROTATE:
          if (this.enableZoom === false && this.enableRotate === false) return;

          handleTouchMoveDollyRotate(event);

          this.update();

          break;

        default:
          state = STATE.NONE;
      }
    }

    function addPointer(event: FakePointerEvent) {
      pointers.push(event);
    }

    function removePointer(event: FakePointerEvent) {
      delete pointerPositions[event.detail.pointerId];

      for (let i = 0; i < pointers.length; i++) {
        if (pointers[i].detail.pointerId == event.detail.pointerId) {
          pointers.splice(i, 1);
          return;
        }
      }
    }

    function trackPointer(event: FakePointerEvent) {
      let position = pointerPositions[event.detail.pointerId];

      if (position === undefined) {
        position = new Vector2();
        pointerPositions[event.detail.pointerId] = position;
      }

      position.set(event.detail.pageX, event.detail.pageY);
    }

    function getSecondPointerPosition(event: FakePointerEvent) {
      const pointer =
        event.detail.pointerId === pointers[0].detail.pointerId
          ? pointers[1]
          : pointers[0];

      return pointerPositions[pointer.detail.pointerId];
    }

    //@ts-ignore
    this.domElement.addEventListener("pointerdown", onPointerDown);
    //@ts-ignore
    this.domElement.addEventListener("pointercancel", onPointerCancel);
    //@ts-ignore
    this.domElement.addEventListener("wheel", onMouseWheel, {
      passive: false,
    });

    // force an update at start
    this.update();
  }
}
