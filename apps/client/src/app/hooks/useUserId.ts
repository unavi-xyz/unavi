import { useSubscribeValue } from "../../editor/hooks/useSubscribeValue";
import { useAppStore } from "../store";

export function useUserId() {
  const engine = useAppStore((state) => state.engine);

  const playerId$ = engine?.networkingInterface.playerId$;
  const playerId = useSubscribeValue(playerId$);

  return playerId;
}
