import { RefObject, useRef } from "react";

export function useInterp(valueRef: RefObject<number>) {
  const interpolatedRef = useRef(valueRef.current);

  return interpolatedRef;
}
