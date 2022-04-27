import { useEthersStore } from "../../../../helpers/ethers/store";
import { useLensStore } from "../../../../helpers/lens/store";

import SignInButton from "./SignInButton";
import ProfileButton from "./ProfileButton";

export default function BottomButton() {
  const address = useEthersStore((state) => state.address);
  const handle = useLensStore((state) => state.handle);

  if (address || handle) return <ProfileButton />;
  return <SignInButton />;
}
