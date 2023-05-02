import { Node } from "@gltf-transform/core";
import { useState } from "react";

import { getNewProjectAssetUpload } from "@/app/api/projects/[id]/assets/helper";

import FileInput from "../../../ui/FileInput";
import { useAvatar } from "../../hooks/useExtension";
import { useSubscribe } from "../../hooks/useSubscribe";
import { useStudio } from "../Studio";
import ComponentMenu from "./ComponentMenu";
import MenuRows from "./ui/MenuRows";

interface Props {
  projectId: string;
  node: Node;
}

export default function AvatarComponent({ projectId, node }: Props) {
  const { mode } = useStudio();
  const avatar = useAvatar(node);
  const uri = useSubscribe(avatar, "URI");
  const equippable = useSubscribe(avatar, "Equippable");

  const [loading, setLoading] = useState(false);

  if (uri === undefined) return null;

  const fileName = avatar ? avatar.getName() : undefined;
  const uriName = uri ? `File ${uri.split("/").pop()?.substring(0, 6)}` : undefined;

  return (
    <ComponentMenu
      title="Avatar"
      onRemove={() => {
        avatar?.dispose();
      }}
    >
      <FileInput
        displayName={fileName || uriName}
        placeholder="Upload VRM File"
        accept=".vrm"
        disabled={loading || mode === "play"}
        onChange={async (e) => {
          const file = e.target.files?.[0];
          if (!avatar || !file || loading) return;

          avatar.setName(file.name);
          setLoading(true);

          try {
            const { url, assetId } = await getNewProjectAssetUpload(projectId);

            const res = await fetch(url, {
              body: file,
              headers: {
                "Content-Type": file.type,
                "x-amz-acl": "public-read",
              },
              method: "PUT",
            });
            if (!res.ok) return;

            avatar.setURI(`/assets/${assetId}`);
          } catch (e) {
            console.error(e);
            avatar.setName("");
          }

          setLoading(false);
        }}
      />

      <MenuRows titles={["Equippable"]}>
        <input
          type="checkbox"
          disabled={loading || mode === "play"}
          checked={equippable ?? false}
          onChange={(e) => {
            avatar?.setEquippable(e.target.checked);
          }}
          className="h-full w-4"
        />
      </MenuRows>
    </ComponentMenu>
  );
}
