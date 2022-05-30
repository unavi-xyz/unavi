import { Debug } from "@react-three/cannon";

interface Props {
  enabled: boolean;
  children: JSX.Element;
}

export default function ToggleDebug({ enabled, children }: Props) {
  if (enabled) return <Debug>{children}</Debug>;
  return children;
}
