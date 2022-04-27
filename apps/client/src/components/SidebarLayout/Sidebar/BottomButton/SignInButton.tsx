import { useState } from "react";
import SignInPage from "./SignInPage";
import { Dialog } from "../../../base";

export default function SignInButton() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Dialog open={open} setOpen={setOpen}>
        <SignInPage />
      </Dialog>

      <div
        onClick={() => setOpen(true)}
        className="bg-black text-white mx-8 py-2 rounded-full flex justify-center cursor-pointer"
      >
        Sign in
      </div>
    </>
  );
}
