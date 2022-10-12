import { IoMdPerson } from "react-icons/io";

import TextField from "../../ui/TextField";
import { useUserId } from "../hooks/useUserId";
import { useAppStore } from "../store";

export default function UserPage() {
  const displayName = useAppStore((state) => state.displayName);
  const userId = useUserId();

  return (
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
  );
}
