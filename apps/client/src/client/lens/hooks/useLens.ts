import { useContext } from "react";

import { LensContext } from "../context";

export function useLens() {
  return useContext(LensContext);
}
