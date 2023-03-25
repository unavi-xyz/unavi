import { useContext } from "react";

import { ClientContext } from "../Client";

export function useMic() {
  const { micEnabled, setMicEnabled, micTrack, setMicTrack } = useContext(ClientContext);
  return { micEnabled, setMicEnabled, micTrack, setMicTrack };
}
