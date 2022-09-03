import { useConnectModal } from "@rainbow-me/rainbowkit";

import Button from "../../../ui/base/Button";

export default function LoginButton() {
  const { openConnectModal } = useConnectModal();

  return (
    <Button variant="filled" onClick={openConnectModal}>
      Login
    </Button>
  );
}
