import { IoMdPerson } from "react-icons/io";

import { useLens } from "../../client/lens/hooks/useLens";
import LoginButton from "../../home/layouts/NavbarLayout/LoginButton";
import Button from "../../ui/Button";
import ButtonFileInput from "../../ui/ButtonFileInput";
import { useAppStore } from "../store";
import UserIdentityButton from "./UserIdentityButton";

export default function UserPage() {
  const displayName = useAppStore((state) => state.displayName);
  const customAvatar = useAppStore((state) => state.customAvatar);
  const { handle } = useLens();

  return (
    <div className="space-y-4">
      <div className="flex w-full justify-center">
        <IoMdPerson className="text-5xl" />
      </div>

      <div className="space-y-1">
        <div className="flex justify-center">
          {handle ? (
            <UserIdentityButton />
          ) : (
            <LoginButton fullWidth rounded="large" />
          )}
        </div>
      </div>

      {!handle && (
        <div className="space-y-1">
          <input
            type="text"
            placeholder="Enter your name..."
            value={displayName ?? ""}
            onChange={(e) => {
              useAppStore.setState({
                displayName: e.target.value,
                didChangeName: true,
              });
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

            useAppStore.setState({
              customAvatar: url,
              didChangeAvatar: true,
            });
          }}
        >
          {customAvatar ? "Change Avatar" : "Upload Avatar"}
        </ButtonFileInput>

        {customAvatar && (
          <Button
            color="error"
            rounded="large"
            fullWidth
            onClick={() => {
              useAppStore.setState({
                customAvatar: null,
                didChangeAvatar: true,
              });
            }}
          >
            Remove Avatar
          </Button>
        )}
      </div>
    </div>
  );
}
