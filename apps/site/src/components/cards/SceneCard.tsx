import { useEffect, useState } from "react";
import { Typography } from "@mui/material";
import { useIdenticon } from "ui";

import BasicCard from "./BasicCard";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const identicon = useIdenticon(id);

  const [preview, setPreview] = useState("");
  const [name, setName] = useState<null | string>();

  useEffect(() => {
    setPreview(localStorage.getItem(`${id}-preview`));

    const newName = localStorage.getItem(`${id}-name`);
    setName(newName?.length > 0 ? newName : id);
  }, [id]);

  return (
    <BasicCard href={`/home/scene/${id}`} image={preview ?? identicon}>
      <Typography>🚧 {name}</Typography>
    </BasicCard>
  );
}
