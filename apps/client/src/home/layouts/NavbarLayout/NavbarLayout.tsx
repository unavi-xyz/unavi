import Navbar from "./Navbar";

interface Props {
  children: React.ReactNode;
}

export default function NavbarLayout({ children }: Props) {
  return (
    <div className="h-full overflow-hidden">
      <div className="z-10 h-14 w-full">
        <Navbar />
      </div>
      <div className="h-full w-full snap-y snap-mandatory overflow-y-scroll">
        {children}
      </div>
    </div>
  );
}

export function getNavbarLayout(children: React.ReactNode) {
  return <NavbarLayout>{children}</NavbarLayout>;
}
