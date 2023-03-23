import Image from "next/image";
import Link from "next/link";

import Logo from "../../public/images/Logo.png";
import ClientButtons from "./ClientButtons";
import NavbarTab from "./NavbarTab";

export default function NavbarLayout({ children }: { children: React.ReactNode }) {
  return (
    <div>
      <div className="absolute z-20 h-14 w-full">
        <nav
          className="flex h-full w-full justify-center bg-white"
          style={{ paddingLeft: "calc(100vw - 100%)" }}
        >
          <div className="max-w-content mx-4 flex justify-between lg:grid lg:grid-cols-3">
            <div className="flex items-center">
              <Link
                href="/"
                className="relative aspect-square h-9 cursor-pointer rounded-full outline-neutral-400"
              >
                <Image src={Logo} alt="logo" priority width={36} height={36} />
              </Link>
            </div>

            <div className="flex items-center justify-center space-x-1 lg:space-x-4">
              <div>
                <NavbarTab text="Explore" href="/explore" />
              </div>

              <div className="hidden lg:block">
                <NavbarTab text="Create" href="/create" />
              </div>
            </div>

            <div className="flex items-center justify-end">
              <ClientButtons />
            </div>
          </div>
        </nav>
      </div>

      <div className="h-screen pt-14">{children}</div>
    </div>
  );
}
