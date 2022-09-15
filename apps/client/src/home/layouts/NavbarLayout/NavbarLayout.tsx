import Navbar from "./Navbar";

interface Props {
  children: React.ReactNode;
}

export default function NavbarLayout({ children }: Props) {
  return (
    <div className="flex h-full snap-y snap-mandatory flex-col items-center overflow-y-scroll">
      <div className="fixed z-30 h-14 w-full md:pr-2">
        <Navbar />
      </div>
      <div className="h-full w-full pt-14">{children}</div>
    </div>
  );
}

export function getNavbarLayout(children: React.ReactNode) {
  return <NavbarLayout>{children}</NavbarLayout>;
}
