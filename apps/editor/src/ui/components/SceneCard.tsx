import { useEffect, useState } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Stack,
  Typography,
} from "@mui/material";
import Link from "next/link";
import { useIdenticon } from "ui";

import SceneActions from "./SceneActions";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const identicon = useIdenticon(id);

  const [hover, setHover] = useState(false);
  const [deleted, setDeleted] = useState(false);
  const [preview, setPreview] = useState("");
  const [name, setName] = useState<null | string>();

  useEffect(() => {
    setPreview(localStorage.getItem(`${id}-preview`));
    setName(localStorage.getItem(`${id}-name`));
  }, [id]);

  if (deleted) return <></>;

  return (
    <Grid item xs={12} sm={6}>
      <Card variant="outlined">
        <Link href={`/scene/${id}`} passHref>
          <CardActionArea
            onMouseOver={() => setHover(true)}
            onMouseOut={() => setHover(false)}
          >
            <CardMedia
              component="img"
              height="140px"
              image={preview ?? identicon}
            />
            <CardContent sx={{ p: 1, borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              <Stack
                direction="row"
                justifyContent="space-between"
                alignItems="center"
              >
                <Typography>{name ?? id}</Typography>
                <span style={{ visibility: hover ? "visible" : "hidden" }}>
                  <SceneActions id={id} setDeleted={setDeleted} />
                </span>
              </Stack>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
