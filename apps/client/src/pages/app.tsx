import { useAppHotkeys } from "../components/app/helpers/hooks/useAppHotkeys";

import AppCanvas from "../components/app/AppCanvas/AppCanvas";
import AppOverlay from "../components/app/AppOverlay/AppOverlay";
import SocketProvider from "../components/app/SocketProvider";

export default function App() {
  useAppHotkeys();

  return (
    <div className="h-full">
      <SocketProvider>
        <AppCanvas />
        <AppOverlay />
      </SocketProvider>
    </div>
  );
}
