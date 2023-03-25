import { useContext } from "react";

import { ClientContext } from "../components/Client";

export function useLoading() {
  const { loadingProgress, loadingText } = useContext(ClientContext);
  return { progress: loadingProgress, text: loadingText };
}
