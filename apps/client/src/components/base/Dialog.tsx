import React, { useState, useEffect, useRef, useContext } from "react";
import { createPortal } from "react-dom";

const DialogContext = React.createContext({ close: () => {} });

interface Props {
  open: boolean;
  onClose: () => void;
  children: React.ReactNode;
}

export default function Dialog({ open, onClose, children }: Props) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const scrimRef = useRef<HTMLDivElement>(null);

  const [visible, setVisible] = useState(false);

  useEffect(() => {
    //add a delay before unmounting the menu so we can show animations
    const timeout = setTimeout(() => setVisible(open), 200);

    //dont add the delay on open
    if (open) {
      setVisible(true);
      clearTimeout(timeout);
    }

    //open / close animation
    if (open) {
      setTimeout(() => {
        dialogRef.current?.classList.remove("scale-75");
        dialogRef.current?.classList.remove("opacity-0");
        scrimRef.current?.classList.remove("opacity-0");
      }, 10);
    } else {
      dialogRef.current?.classList.add("opacity-0");
      dialogRef.current?.classList.add("scale-75");
      scrimRef.current?.classList.add("opacity-0");
    }

    return () => clearTimeout(timeout);
  }, [open]);

  if (!visible) return null;

  return createPortal(
    <div
      ref={scrimRef}
      onMouseDown={onClose}
      className="fixed top-0 left-0 bg-black bg-opacity-50 w-screen
                 h-screen flex flex-col justify-center z-50 opacity-0
                 transition-opacity duration-200 ease-in-out"
    >
      <dialog
        ref={dialogRef}
        open
        onMouseDown={(e) => e.stopPropagation()}
        className="rounded-3xl p-12 mx-auto w-full max-w-xl flex flex-col space-y-4
                   transition duration-200 ease-in-out scale-75 opacity-0 bg-surface
                   text-onSurface"
      >
        <DialogContext.Provider value={{ close: onClose }}>
          {children}
        </DialogContext.Provider>
      </dialog>
    </div>,
    document.body
  );
}

export function useCloseDialog() {
  const { close } = useContext(DialogContext);
  return close;
}
