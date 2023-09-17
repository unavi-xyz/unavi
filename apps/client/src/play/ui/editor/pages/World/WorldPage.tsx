import { syncedStore } from "@unavi/engine";
import { useSnapshot } from "valtio";

import { usePlayStore } from "@/app/play/playStore";
import ImageInput from "@/src/ui/ImageInput";
import TextAreaDark from "@/src/ui/TextAreaDark";
import TextFieldDark from "@/src/ui/TextFieldDark";
import { cropImage } from "@/src/utils/cropImage";

import { useSave } from "../../../../hooks/useSave";
import PanelPage from "../PanelPage";

export default function WorldPage() {
  const worldId = usePlayStore((state) => state.worldId);
  const image = usePlayStore((state) => state.metadata?.image);

  const snap = useSnapshot(syncedStore);

  const { save, saving } = useSave();

  const placeholder = worldId.type === "id" ? `World ${worldId.value}` : "";

  return (
    <PanelPage title="World">
      <ImageInput
        src={image}
        onChange={async (e) => {
          const file = e.target.files?.[0];
          if (!file) return;

          const fileURL = URL.createObjectURL(file);
          const cropped = await cropImage(fileURL);
          const croppedURL = URL.createObjectURL(cropped);

          usePlayStore.setState((prev) => ({
            metadata: {
              ...prev.metadata,
              image: croppedURL,
            },
          }));

          URL.revokeObjectURL(fileURL);
        }}
        dark
        className="h-40 w-full rounded-lg object-cover"
      />

      <TextFieldDark
        label="Title"
        name="title"
        placeholder={placeholder}
        value={snap.metadata.title}
        onChange={(e) => {
          syncedStore.metadata.title = e.target.value;
        }}
      />

      <TextAreaDark
        label="Description"
        name="description"
        placeholder={"Wow, this world is so cool!"}
        value={snap.metadata.description}
        onChange={(e) => {
          syncedStore.metadata.description = e.target.value;
        }}
        className="max-h-40"
      />

      <div className="absolute inset-x-4 bottom-4">
        <button
          onClick={save}
          disabled={saving}
          className={`w-full rounded-full bg-neutral-800 py-2 font-bold transition ${saving
              ? "opacity-50"
              : "hover:scale-105 hover:bg-neutral-700 active:scale-100 active:opacity-90"
            }`}
        >
          Save
        </button>
      </div>
    </PanelPage>
  );
}
