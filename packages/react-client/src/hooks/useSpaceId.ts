import { useContext } from "react";

import { ClientContext } from "../components/Client";

export function useSpaceId() {
  return useContext(ClientContext).spaceId;
}
