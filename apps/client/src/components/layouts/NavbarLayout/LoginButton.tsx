import { useState } from "react";

import Dialog from "../../base/Dialog";
import LoginPage from "./LoginPage";

export default function LoginButton() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Dialog open={open} onClose={() => setOpen(false)}>
        <LoginPage />
      </Dialog>

      <button
        onClick={() => setOpen(true)}
        className="cursor-pointer bg-black text-white text-sm font-bold rounded-full px-6 py-1.5"
      >
        Login
      </button>
    </>
  );
}
