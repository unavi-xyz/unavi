import { MouseEvent, useEffect, useRef, useState } from "react";
import { BsMicFill, BsMicMuteFill } from "react-icons/bs";

import { appManager, useStore } from "../helpers/store";

export default function Mic() {
  const buttonRef = useRef<HTMLDivElement>();

  const muted = useStore((state) => state.muted);

  const [track, setTrack] = useState<MediaStreamTrack>(undefined);

  async function handleClick(e: MouseEvent) {
    e.stopPropagation();

    if (!track) {
      const newTrack = await appManager.getUserMedia();
      setTrack(newTrack);
      useStore.setState({ muted: false });
      return;
    }

    useStore.setState({ muted: !muted });
  }

  useEffect(() => {
    if (!track) return;
    track.enabled = !muted;
  }, [muted, track]);

  useEffect(() => {
    if (!track) return;
    const stream = new MediaStream();
    stream.addTrack(track);

    const ctx = new AudioContext();
    const source = ctx.createMediaStreamSource(stream);
    const analyzer = ctx.createAnalyser();

    source.connect(analyzer);
    source.connect(ctx.destination);

    const frequencyData = new Uint8Array(analyzer.frequencyBinCount);

    let stopLoop = false;

    function render() {
      analyzer.getByteFrequencyData(frequencyData);

      const total = frequencyData.reduce((prev, current) => prev + current);
      const average = total / frequencyData.length;
      const percent = Math.min(average / 50, 1) * 100;

      if (buttonRef.current) {
        const showBorder = percent > 5;

        if (showBorder) buttonRef.current.style.borderWidth = "2px";
        else buttonRef.current.style.borderWidth = "0";
      }

      if (!stopLoop) requestAnimationFrame(render);
    }

    requestAnimationFrame(render);

    return () => {
      stopLoop = true;
    };
  }, [track]);

  return (
    <div className="flex w-full justify-center p-8">
      <div
        ref={buttonRef}
        onClick={handleClick}
        className="relative rounded-xl w-12 h-12 bg-white cursor-pointer hover:bg-neutral-200
                   z-10 flex items-center justify-center overflow-hidden border-primary"
      >
        {muted ? (
          <BsMicMuteFill className="z-10 text-lg text-red-600" />
        ) : (
          <BsMicFill className="z-10 text-lg" />
        )}
      </div>
    </div>
  );
}
