import { Debug } from "@react-three/cannon";

interface Props {
  enabled: boolean;
  children: React.ReactNode;
}

export default function ToggleDebug({ enabled, children }: Props) {
  if (enabled) return <Debug>{children}</Debug>;
  return <group>{children}</group>;
}
