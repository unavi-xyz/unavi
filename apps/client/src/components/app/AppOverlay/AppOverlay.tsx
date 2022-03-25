import Chat from "./Chat";

export default function AppOverlay() {
  return (
    <div>
      <div className="crosshair" />

      <div className="absolute w-full h-full top-0 left-0 flex flex-col justify-end">
        <Chat />
      </div>
    </div>
  );
}
