const ACTIVE = false;

export default function Crosshair() {
  return (
    <>
      {ACTIVE ? (
        <>
          <div className="pointer-events-none absolute left-1/2 top-1/2 z-30 h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 rounded-sm backdrop-blur backdrop-invert" />

          <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
            <div className="z-20 h-1.5 w-1.5 scale-125 animate-ping rounded-sm backdrop-invert" />
          </div>
        </>
      ) : (
        <div className="pointer-events-none absolute left-1/2 top-1/2 z-30 h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full backdrop-blur backdrop-invert" />
      )}
    </>
  );
}
