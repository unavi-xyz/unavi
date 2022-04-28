import { useState, useEffect } from "react";

import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import ProfileMenu from "./ProfileMenu";

export default function ProfileButton() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    //if the user clicks outside of the profile menu, close it
    function handleClickOutside(event: MouseEvent) {
      if (open) setOpen(false);
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [open]);

  return (
    <div className="flex items-center space-x-2">
      <div className="relative">
        <div
          onMouseDown={(e) => {
            e.stopPropagation();
            setOpen((prev) => !prev);
          }}
          className="w-9 h-9 rounded-full border cursor-pointer"
        >
          <ViewerProfilePicture circle />
        </div>

        <div
          onMouseDown={(e) => e.stopPropagation()}
          className="absolute mt-1 right-0"
        >
          <ProfileMenu open={open} />
        </div>
      </div>
    </div>
  );
}
