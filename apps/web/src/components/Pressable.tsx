import { useState } from "react";

export default function Pressable({ children }) {
  const [clicked, setClicked] = useState(false);

  const css = clicked ? "translate-y-px" : "";

  return (
    <div
      onMouseDown={() => setClicked(true)}
      onMouseUp={() => setClicked(false)}
      onMouseOut={() => setClicked(false)}
      className={css}
    >
      {children}
    </div>
  );
}
