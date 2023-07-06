import { usePlayStore } from "@/app/play/store";
import ImageInput from "@/src/ui/ImageInput";
import TextAreaDark from "@/src/ui/TextAreaDark";
import TextFieldDark from "@/src/ui/TextFieldDark";

import { useSave } from "../../hooks/useSave";

export default function WorldPage() {
  const worldId = usePlayStore((state) => state.worldId);
  const image = usePlayStore((state) => state.metadata.info?.image);
  const title = usePlayStore((state) => state.metadata.info?.title);
  const description = usePlayStore((state) => state.metadata.info?.description);

  const save = useSave();

  const placeholder =
    worldId.type === "id" ? `World ${worldId.value.slice(0, 6)}` : "";

  return (
    <div className="flex h-full flex-col justify-between">
      <div className="space-y-4">
        <ImageInput
          src={image}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (!file) return;

            const reader = new FileReader();

            reader.onload = (e) => {
              const image = e.target?.result as string;

              usePlayStore.setState((prev) => ({
                metadata: {
                  ...prev.metadata,
                  info: {
                    ...prev.metadata.info,
                    image,
                  },
                },
              }));
            };

            reader.readAsDataURL(file);
          }}
          dark
          className="h-40 w-full rounded-lg object-cover"
        />

        <TextFieldDark
          label="Title"
          name="title"
          placeholder={placeholder}
          value={title}
          onChange={(e) => {
            usePlayStore.setState((prev) => ({
              metadata: {
                ...prev.metadata,
                info: {
                  ...prev.metadata.info,
                  name: e.target.value,
                },
              },
            }));
          }}
        />

        <TextAreaDark
          label="Description"
          name="description"
          placeholder={"Wow, this world is so cool!"}
          value={description}
          onChange={(e) => {
            usePlayStore.setState((prev) => ({
              metadata: {
                ...prev.metadata,
                info: {
                  ...prev.metadata.info,
                  description: e.target.value,
                },
              },
            }));
          }}
          className="max-h-40"
        />
      </div>

      <button
        onClick={save}
        className="rounded-full bg-neutral-800 py-2 font-bold transition hover:scale-105 hover:bg-neutral-700 active:scale-100 active:opacity-90"
      >
        Save
      </button>
    </div>
  );
}
