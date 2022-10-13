import { useConnectModal } from "@rainbow-me/rainbowkit";

import Button from "../../../ui/Button";

interface Props {
  fullWidth?: boolean;
}

export default function LoginButton({ fullWidth = false }: Props) {
  const { openConnectModal } = useConnectModal();

  return (
    <Button variant="filled" fullWidth={fullWidth} onClick={openConnectModal}>
      Sign in
    </Button>
  );
}
