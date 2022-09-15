import React, {
  ReactPortal,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";

const DialogContext = React.createContext({ close: () => {} });

interface Props {
  open: boolean;
  onClose?: () => void;
  children: React.ReactNode;
}

export default function Dialog({
  open,
  onClose = () => {},
  children,
}: Props): ReactPortal | null {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const scrimRef = useRef<HTMLDivElement>(null);

  const [visible, setVisible] = useState(false);

  useEffect(() => {
    // Add a delay before unmounting the menu so we can show animations
    const timeout = setTimeout(() => setVisible(open), 200);

    // Don't add the delay on open
    if (open) {
      setVisible(true);
      clearTimeout(timeout);
    }

    // Open / close animation
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
      className="fixed top-0 left-0 z-50 flex h-screen
                 w-screen flex-col justify-center bg-black bg-opacity-30 opacity-0
                 transition-opacity duration-200 ease-in-out"
    >
      <dialog
        ref={dialogRef}
        open
        onMouseDown={(e) => e.stopPropagation()}
        className="bg-surface text-onSurface flex w-full max-w-xl scale-75 flex-col
                   space-y-4 rounded-3xl p-10 opacity-0 drop-shadow-lg transition
                   duration-200 ease-in-out"
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
