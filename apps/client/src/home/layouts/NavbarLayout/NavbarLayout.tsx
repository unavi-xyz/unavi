import Navbar from "./Navbar";

interface Props {
  children: React.ReactNode;
}

export default function NavbarLayout({ children }: Props) {
  return (
    <>
      <div className="absolute z-20 h-14 w-full">
        <Navbar />
      </div>

      <div className="h-full w-full">
        <div className="pt-14">{children}</div>
      </div>
    </>
  );
}

export function getNavbarLayout(children: React.ReactNode) {
  return <NavbarLayout>{children}</NavbarLayout>;
}
