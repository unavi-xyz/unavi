import { useContext } from "react";

import { ClientContext } from "../Client";

export function useEngine() {
  return useContext(ClientContext).engine;
}
