import { IoMdPerson } from "react-icons/io";

import { useAppStore } from "../../app/store";
import { useSession } from "../../client/auth/useSession";
import ProfileButton from "../../home/layouts/NavbarLayout/ProfileButton";
import SignInButton from "../../home/layouts/NavbarLayout/SignInButton";
import Button from "../../ui/Button";
import ButtonFileInput from "../../ui/ButtonFileInput";

export default function UserPage() {
  const playerId = useAppStore((state) => state.playerId);
  const { data: session } = useSession();

  const nickname = useAppStore((state) => state.nickname);
  const avatar = useAppStore((state) => state.avatar);

  const guestName =
    playerId == null || playerId === undefined
      ? "Guest"
      : `Guest 0x${playerId.toString(16).padStart(2, "0")}`;

  return (
    <div className="space-y-4">
      <div className="flex w-full justify-center">
        <IoMdPerson className="text-5xl" />
      </div>

      <div className="space-y-1">
        <div className="flex justify-center">
          {session?.address ? <ProfileButton /> : <SignInButton />}
        </div>
      </div>

      {!session?.address && (
        <div className="space-y-1">
          <input
            type="text"
            placeholder={guestName}
            value={nickname ?? ""}
            onChange={(e) => {
              useAppStore.setState({ didChangeName: true, nickname: e.target.value });
            }}
            className="h-full w-full rounded-xl bg-neutral-200/80 px-3 py-2 text-center outline-none ring-neutral-500/80 transition hover:ring-1 focus:bg-neutral-200 focus:ring-1"
          />
        </div>
      )}

      <div className="space-y-1">
        <ButtonFileInput
          accept=".vrm"
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (!file) return;

            const blob = new Blob([file], { type: "model/gltf-binary" });
            const url = URL.createObjectURL(blob);

            useAppStore.setState({ didChangeAvatar: true, avatar: url });
          }}
        >
          {avatar ? "Change Avatar" : "Upload Avatar"}
        </ButtonFileInput>

        {avatar && (
          <Button
            onClick={() => {
              useAppStore.setState({ didChangeAvatar: true, avatar: null });
            }}
          >
            Remove Avatar
          </Button>
        )}
      </div>
    </div>
  );
}
