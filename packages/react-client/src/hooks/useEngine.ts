import { useContext } from "react";

import { ClientContext } from "../components/Client";

export function useEngine() {
  return useContext(ClientContext).engine;
}
