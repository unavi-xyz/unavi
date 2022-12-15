import { useConnectModal } from "@rainbow-me/rainbowkit";

import Button from "../../../ui/Button";

interface Props {
  fullWidth?: boolean;
  rounded?: "full" | "large" | "small";
}

export default function LoginButton({ fullWidth = false, rounded }: Props) {
  const { openConnectModal } = useConnectModal();

  return (
    <Button variant="filled" fullWidth={fullWidth} rounded={rounded} onClick={openConnectModal}>
      Sign in
    </Button>
  );
}
