import { Dispatch, SetStateAction, useContext, useEffect } from "react";
import { CeramicContext } from "ceramic";

import MetamaskFox from "./MetamaskFox";
import Dialog from "../../../Dialog/Dialog";
import RichButton from "../../../RichButton";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}
export default function SignInDialog({ open, setOpen }: Props) {
  const { authenticated, connect } = useContext(CeramicContext);

  useEffect(() => {
    if (authenticated) setOpen(false);
  }, [authenticated, setOpen]);

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="flex flex-col items-center">
        <h1 className="text-3xl flex justify-center">Sign in</h1>
        <p className="text-lg flex justify-center">
          Choose how you want to sign in
        </p>
      </div>

      <RichButton
        title="Metamask"
        description="A browser extension"
        image={<MetamaskFox width="100px" />}
        onClick={connect}
      />
    </Dialog>
  );
}
