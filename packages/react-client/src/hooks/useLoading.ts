import { useContext } from "react";

import { ClientContext } from "../Client";

export function useLoading() {
  const { loadingProgress, loadingText } = useContext(ClientContext);
  return { progress: loadingProgress, text: loadingText };
}
