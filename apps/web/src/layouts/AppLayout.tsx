import Overlay from "../components/Overlay/Overlay";

export default function AppLayout({ children }) {
  return (
    <div className="h-full">
      <Overlay />
      {children}
    </div>
  );
}
