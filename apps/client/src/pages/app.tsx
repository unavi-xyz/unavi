import { useAppHotkeys } from "../components/app/helpers/hooks/useAppHotkeys";

import AppCanvas from "../components/app/AppCanvas/AppCanvas";
import AppOverlay from "../components/app/AppOverlay/AppOverlay";
import MultiplayerProvider from "../components/app/MultiplayerProvider";

export default function App() {
  useAppHotkeys();

  return (
    <div className="h-full">
      <MultiplayerProvider>
        <AppCanvas />
        <AppOverlay />
      </MultiplayerProvider>
    </div>
  );
}
