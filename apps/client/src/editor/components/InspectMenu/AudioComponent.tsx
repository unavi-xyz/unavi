import { Node } from "@gltf-transform/core";
import { AudioEmitterDistanceModel, AudioEmitterType } from "@unavi/gltf-extensions";
import { useState } from "react";

import { getNewProjectAssetUpload } from "@/app/api/projects/[id]/assets/helper";

import FileInput from "../../../ui/FileInput";
import { useAudioEmitter } from "../../hooks/useExtension";
import { useSubscribe } from "../../hooks/useSubscribe";
import { useEditor } from "../Editor";
import EditorInput from "../ui/EditorInput";
import SelectMenu from "../ui/SelectMenu";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  node: Node;
  projectId: string;
}

export default function AudioComponent({ node, projectId }: Props) {
  const { mode } = useEditor();

  const audioEmitter = useAudioEmitter(node);

  const sources = useSubscribe(audioEmitter, "Sources") ?? [];
  const type = useSubscribe(audioEmitter, "Type");
  const gain = useSubscribe(audioEmitter, "Gain");
  const distanceModel = useSubscribe(audioEmitter, "DistanceModel");
  const maxDistance = useSubscribe(audioEmitter, "MaxDistance");
  const refDistance = useSubscribe(audioEmitter, "RefDistance");
  const rolloffFactor = useSubscribe(audioEmitter, "RolloffFactor");

  const source = sources[0] ?? null;
  const sourceName = useSubscribe(source, "Name");
  const audio = useSubscribe(source, "Audio");
  const uri = useSubscribe(audio ?? null, "URI");
  const autoPlay = useSubscribe(source, "AutoPlay") ?? false;
  const loop = useSubscribe(source, "Loop") ?? false;

  const [loadingFileUpload, setLoadingFileUpload] = useState(false);

  if (!audioEmitter) return null;

  return (
    <ComponentMenu
      title="Audio"
      onRemove={() => {
        audioEmitter.dispose();
      }}
    >
      <FileInput
        displayName={uri ? sourceName : undefined}
        placeholder="Upload MP3 File"
        accept=".mp3"
        disabled={loadingFileUpload || mode === "play"}
        onChange={async (e) => {
          const file = e.target.files?.[0];
          if (!source || !file) return;

          source.setName(file.name);
          setLoadingFileUpload(true);

          const audioData = source.getAudio();
          if (!audioData) return;

          try {
            const { url, assetId } = await getNewProjectAssetUpload(projectId);

            const res = await fetch(url, {
              method: "PUT",
              body: file,
              headers: {
                "Content-Type": file.type,
                "x-amz-acl": "public-read",
              },
            });
            if (!res.ok) return;

            audioData.setURI(`/assets/${assetId}`);
          } catch (e) {
            console.error(e);
            source.setName("");
          }

          setLoadingFileUpload(false);
        }}
      />

      {source ? (
        <MenuRows titles={["Auto Play", "Loop"]}>
          <EditorInput
            name="Auto Play"
            type="checkbox"
            checked={autoPlay}
            onChange={(e) => {
              source.setAutoPlay(e.target.checked);
            }}
          />

          <EditorInput
            name="Loop"
            type="checkbox"
            checked={loop}
            onChange={(e) => {
              source.setLoop(e.target.checked);
            }}
          />
        </MenuRows>
      ) : null}

      <MenuRows titles={["Type", "Gain"]}>
        <SelectMenu
          name="Type"
          options={["global", "positional"]}
          value={type ?? "global"}
          onChange={(e) => {
            audioEmitter.setType(e.target.value as AudioEmitterType);
          }}
        />

        <EditorInput
          name="Gain"
          type="number"
          min={0}
          max={1}
          step={0.01}
          value={gain ?? 1}
          onChange={(e) => {
            const clampedValue = Math.min(Math.max(parseFloat(e.target.value), 0), 1);
            audioEmitter.setGain(clampedValue);
          }}
        />
      </MenuRows>

      {type === "positional" ? (
        <MenuRows titles={["Distance Model", "Max Distance", "Ref Distance", "Rolloff Factor"]}>
          <SelectMenu
            name="Distance Model"
            options={["linear", "inverse", "exponential"]}
            value={distanceModel ?? "inverse"}
            onChange={(e) => {
              audioEmitter.setDistanceModel(e.target.value as AudioEmitterDistanceModel);
            }}
          />

          <EditorInput
            name="Max Distance"
            type="number"
            min={0}
            value={maxDistance ?? 1}
            onChange={(e) => {
              audioEmitter.setMaxDistance(parseFloat(e.target.value));
            }}
          />

          <EditorInput
            name="Ref Distance"
            type="number"
            min={0}
            value={refDistance ?? 1}
            onChange={(e) => {
              audioEmitter.setRefDistance(parseFloat(e.target.value));
            }}
          />

          <EditorInput
            name="Rolloff Factor"
            type="number"
            min={0}
            step={0.01}
            value={rolloffFactor ?? 1}
            onChange={(e) => {
              audioEmitter.setRolloffFactor(parseFloat(e.target.value));
            }}
          />
        </MenuRows>
      ) : null}
    </ComponentMenu>
  );
}
