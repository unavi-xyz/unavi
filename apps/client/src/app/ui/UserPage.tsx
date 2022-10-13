import { IoMdPerson } from "react-icons/io";

import Button from "../../ui/Button";
import ButtonFileInput from "../../ui/ButtonFileInput";
import TextField from "../../ui/TextField";
import { useUserId } from "../hooks/useUserId";
import { useAppStore } from "../store";

export default function UserPage() {
  const displayName = useAppStore((state) => state.displayName);
  const customAvatar = useAppStore((state) => state.customAvatar);
  const userId = useUserId();

  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <div className="flex w-full justify-center">
          <IoMdPerson className="text-5xl" />
        </div>

        <div className="text-xl">Name</div>
        <TextField
          outline
          value={displayName ?? `Guest ${userId?.slice(0, 4)}`}
          onChange={(e) => {
            useAppStore.setState({ displayName: e.target.value });
          }}
        />
      </div>

      <div className="space-y-1">
        <div className="text-xl">Avatar</div>

        <ButtonFileInput
          accept=".vrm"
          displayText={customAvatar ? "Change Avatar" : "Upload File"}
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
        />

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
