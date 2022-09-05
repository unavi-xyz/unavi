import { useContext } from "react";

import { LensContext } from "../LensProvider";

export function useLens() {
  return useContext(LensContext);
}
