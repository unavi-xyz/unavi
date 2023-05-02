import "reactflow/dist/style.css";

import { useStudio } from "../Studio";
import NodeGraph from "./NodeGraph";
import NodeGraphHeader from "./NodeGraphHeader";
import Script from "./Script";

export default function ScriptMenu() {
  const { scriptId } = useStudio();

  if (!scriptId) return null;

  return (
    <Script>
      <NodeGraphHeader />
      <NodeGraph key={scriptId} scriptId={scriptId} />
    </Script>
  );
}
