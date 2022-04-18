import { Dispatch, ReactChild, SetStateAction } from "react";
import ReactDOM from "react-dom";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
  children: ReactChild | ReactChild[];
}
export function Dialog({ open, setOpen, children }: Props): JSX.Element {
  if (!open) return null;

  return ReactDOM.createPortal(
    <div
      className="fixed top-0 left-0 bg-black bg-opacity-50 w-screen
                 h-screen flex flex-col justify-center z-50"
      onMouseDown={() => setOpen(false)}
    >
      <dialog
        open
        onMouseDown={(e) => e.stopPropagation()}
        className="rounded-2xl p-12 fade mx-auto w-full max-w-xl flex flex-col space-y-4"
      >
        {children}
      </dialog>
    </div>,
    document.body
  );
}
