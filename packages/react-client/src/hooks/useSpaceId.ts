import { useContext } from "react";

import { ClientContext } from "../Client";

export function useSpaceId() {
  return useContext(ClientContext).spaceId;
}
