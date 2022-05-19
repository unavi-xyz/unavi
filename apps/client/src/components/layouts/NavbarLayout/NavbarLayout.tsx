import Navbar from "./Navbar";

interface Props {
  children: React.ReactNode;
}

export default function NavbarLayout({ children }: Props) {
  return (
    <div className="flex flex-col items-center h-full">
      <div className="w-full h-14 fixed z-50">
        <Navbar />
      </div>
      <div className="w-full h-full pt-14">{children}</div>
    </div>
  );
}
