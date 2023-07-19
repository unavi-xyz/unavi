import { useSceneStore } from "@unavi/react-client";

import TreeItem from "./TreeItem";

export default function SceneTree() {
  const rootId = useSceneStore((state) => state.rootId);

  if (!rootId) return null;

  return <TreeItem id={rootId} />;
}
