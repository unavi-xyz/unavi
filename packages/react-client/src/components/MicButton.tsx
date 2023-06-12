import { forwardRef } from "react";

import { useClient } from "../hooks/useClient";

type Props = React.ButtonHTMLAttributes<HTMLButtonElement>;

/**
 * A button for the user to toggle their microphone.
 */
export const MicButton = forwardRef<HTMLButtonElement, Props>((props, ref) => {
  const { engine, micEnabled, micTrack, setMicEnabled, setMicTrack } = useClient();

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
