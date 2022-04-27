import { connectWallet } from "../../../../helpers/ethers/connection";
import { RichButton } from "../../../base";

import MetamaskFox from "./MetamaskFox";

export default function SignInPage() {
  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Sign in</h1>
        <p className="text-lg flex justify-center">
          Choose how you want to sign in
        </p>
      </div>

      <RichButton
        title="Metamask"
        description="A browser extension"
        image={<MetamaskFox width="100px" />}
        onClick={connectWallet}
      />
    </div>
  );
}
