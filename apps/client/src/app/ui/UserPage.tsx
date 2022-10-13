import { IoMdPerson } from "react-icons/io";

import FileInput from "../../ui/FileInput";
import TextField from "../../ui/TextField";
import { useUserId } from "../hooks/useUserId";
import { useAppStore } from "../store";

export default function UserPage() {
  const displayName = useAppStore((state) => state.displayName);
  const userId = useUserId();

  return (
    <div className="space-y-6">
      <div>
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

      <div>
        <div className="text-xl">Avatar</div>
        <FileInput
          accept=".vrm"
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (!file) return;

            const blob = new Blob([file], { type: "model/gltf-binary" });
            const url = URL.createObjectURL(blob);

            useAppStore.setState({ customAvatar: url });
          }}
        />
      </div>
    </div>
  );
}
