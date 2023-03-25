import { useContext } from "react";

import { ClientContext } from "../components/Client";

export function useClient() {
  return useContext(ClientContext);
}
