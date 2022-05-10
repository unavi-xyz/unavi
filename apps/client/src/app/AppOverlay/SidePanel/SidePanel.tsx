import { useState } from "react";
import { BsPeopleFill } from "react-icons/bs";
import { BiWorld } from "react-icons/bi";
import { useAtomValue, useSetAtom } from "jotai";
import { useRouter } from "next/router";
import { useAuth } from "ceramic";

import { useStore } from "../../helpers/store";
import { pageAtom, spaceAtom } from "./helpers/atoms";

import SignInButton from "./SignInButton";
import PlayersPage from "./pages/PlayersPage/PlayersPage";
import SpacePage from "./pages/SpacePage";
import SidePanelButton from "./SidePanelButton";
import UserPage from "./pages/UserPage";

export default function SidePanel() {
  const router = useRouter();
  const spaceId = router.query.space as string;

  const isPointerLocked = useStore((state) => state.isPointerLocked);

  const [open, setOpen] = useState(false);

  const page = useAtomValue(pageAtom);
  const setSpace = useSetAtom(spaceAtom);

  const { authenticated } = useAuth();

  return (
    <div
      className={`z-20 flex transition ${
        isPointerLocked
          ? "translate-x-[600px]"
          : open
          ? "translate-x-0"
          : "translate-x-[600px]"
      }`}
    >
      <div className="flex">
        <div className="flex flex-col m-4 space-y-2">
          {!authenticated && <SignInButton />}

          <SidePanelButton
            name="Players"
            setOpen={setOpen}
            icon={<BsPeopleFill />}
          />

          <div onMouseDown={() => setSpace(spaceId)}>
            <SidePanelButton
              name="Space"
              setOpen={setOpen}
              icon={<BiWorld />}
            />
          </div>
        </div>
      </div>

      <div
        onClick={(e) => {
          e.stopPropagation();
        }}
        className="w-[600px] z-10"
      >
        <div className="bg-white w-full h-full rounded-l-3xl p-8">
          {page === "Players" ? (
            <PlayersPage />
          ) : page === "Space" ? (
            <SpacePage />
          ) : page === "User" ? (
            <UserPage />
          ) : null}
        </div>
      </div>
    </div>
  );
}
