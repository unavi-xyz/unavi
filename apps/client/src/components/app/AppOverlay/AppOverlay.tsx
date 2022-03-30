import Chat from "./Chat";
import Mic from "./Mic";

export default function AppOverlay() {
  return (
    <div>
      <div className="crosshair" />

      <div className="absolute w-full h-full top-0 left-0 flex items-end">
        <div className="w-full">
          <Chat />
        </div>
        <div className="w-full">
          <Mic />
        </div>
        <div className="w-full"></div>
      </div>
    </div>
  );
}
