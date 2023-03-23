import { useContext } from "react";

import { ClientContext } from "../Client";

export function useMic() {
  const { micEnabled, setMicEnabled } = useContext(ClientContext);
  return { micEnabled, setMicEnabled };
}
