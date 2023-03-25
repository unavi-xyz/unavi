import React from "react";

import { useEngine } from "../hooks/useEngine";
import { useMic } from "../hooks/useMic";

type Props = React.ButtonHTMLAttributes<HTMLButtonElement>;

/**
 * A button for the user to toggle their microphone.
 */
export const MicButton = React.forwardRef<HTMLButtonElement, Props>((props, ref) => {
  const engine = useEngine();
  const { micEnabled, micTrack, setMicEnabled, setMicTrack } = useMic();

  async function toggleMic() {
    if (!engine) return;

    // If first time using mic, request permission
    if (!micEnabled && !micTrack) {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });

      const track = stream.getAudioTracks()[0];
      if (!track) throw new Error("No audio track found");

      setMicTrack(track);
    }

    setMicEnabled(!micEnabled);
  }

  return (
    <button ref={ref} onClick={toggleMic} {...props}>
      {props.children}
    </button>
  );
});

MicButton.displayName = "MicButton";
