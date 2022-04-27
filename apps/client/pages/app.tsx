import { useAppHotkeys } from "../src/app/helpers/hooks/useAppHotkeys";

import AppCanvas from "../src/app/AppCanvas/AppCanvas";
import AppOverlay from "../src/app/AppOverlay/AppOverlay";
import SocketProvider from "../src/app/SocketProvider";

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
