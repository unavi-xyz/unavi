import { useState } from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import { useAuth } from "ceramic";

import SignInPage from "../Sidebar/BottomButton/SignInPage";
import NavbarButton from "./NavbarButton";
import { Dialog } from "../../base";

export default function Navbar() {
  const router = useRouter();
  const { authenticated, viewerId } = useAuth();

  const [open, setOpen] = useState(false);

  return (
    <>
      <Dialog open={open} setOpen={setOpen}>
        <SignInPage />
      </Dialog>

      <div className="bg-white w-full h-14 flex items-center justify-between px-10">
        <Link href="/" passHref>
          <div>
            <NavbarButton text="Home" selected={router.asPath === "/"} />
          </div>
        </Link>

        <Link href="/editor" passHref>
          <div>
            <NavbarButton
              text="Editor"
              selected={router.asPath === "/editor"}
            />
          </div>
        </Link>

        {authenticated ? (
          <Link href={`/user/${viewerId}`} passHref>
            <div>
              <NavbarButton
                text="Profile"
                selected={router.asPath === `/user/${viewerId}`}
              />
            </div>
          </Link>
        ) : (
          <div onClick={() => setOpen(true)}>
            <NavbarButton text="Sign In" />
          </div>
        )}
      </div>
    </>
  );
}
