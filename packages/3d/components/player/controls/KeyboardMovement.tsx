import { useRef, useEffect, MutableRefObject } from "react";
import { Vector3 } from "three";

interface Props {
  paused?: boolean;
  direction: MutableRefObject<Vector3>;
  jump: MutableRefObject<boolean>;
}

/**
 * KeyboardMovement gives the player a direction to move by taking
 * input from any source (currently keyboard) and calculating
 * relative direction.
 *
 * Direction is stored as a Vector3 with the following format
 *    x: left/right movement, + for right
 *    y: forward/back movement, + for forwards
 *    z: up/down movement, + for up
 *
 * @param props
 * @constructor
 */
export default function KeyboardMovement({
  paused = false,
  direction,
  jump,
}: Props) {
  const pressedKeys = useRef([false, false, false, false]);

  useEffect(() => {
    // key events
    const calcDirection = () => {
      const press = pressedKeys.current; // [w, a, s, d]
      const yAxis = -1 * Number(press[0]) + Number(press[2]);
      const xAxis = -1 * Number(press[1]) + Number(press[3]);
      return [xAxis, yAxis, 0];
    };

    function updatePressedKeys(e: KeyboardEvent, pressedState: boolean) {
      switch (e.code || e.key) {
        case "KeyW":
        case "ArrowUp":
        case "Numpad8":
          pressedKeys.current[0] = pressedState;
          break;
        case "KeyA":
        case "ArrowLeft":
        case "Numpad4":
          pressedKeys.current[1] = pressedState;
          break;
        case "KeyS":
        case "ArrowDown":
        case "Numpad5":
        case "Numpad2":
          pressedKeys.current[2] = pressedState;
          break;
        case "KeyD":
        case "ArrowRight":
        case "Numpad6":
          pressedKeys.current[3] = pressedState;
          break;
        case "Space":
          jump.current = pressedState;
          break;
        default:
          return;
      }
    }

    function onKeyDown(e: KeyboardEvent) {
      if (e.defaultPrevented) {
        return;
      }

      // We don't want to mess with the browser's shortcuts
      if (e.ctrlKey || e.altKey || e.metaKey) {
        return;
      }

      updatePressedKeys(e, true);

      const [x, y, z] = calcDirection();
      direction.current.set(x, y, z);
    }

    function onKeyUp(e: KeyboardEvent) {
      updatePressedKeys(e, false);

      const [x, y, z] = calcDirection();
      direction.current.set(x, y, z);
    }

    if (paused) {
      direction.current.set(0, 0, 0);
      pressedKeys.current = [false, false, false, false];
      return;
    }

    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("keyup", onKeyUp);

    return () => {
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("keyup", onKeyUp);
    };
  }, [paused, direction, jump]);

  return <></>;
}
