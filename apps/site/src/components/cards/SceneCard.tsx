import { useEffect, useState } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Typography,
} from "@mui/material";
import Link from "next/link";
import { useIdenticon } from "ui";

interface Props {
  id: string;
}

export default function SceneCard({ id }: Props) {
  const identicon = useIdenticon(id);

  const [preview, setPreview] = useState("");
  const [name, setName] = useState<null | string>();

  useEffect(() => {
    setPreview(localStorage.getItem(`${id}-preview`));
    setName(localStorage.getItem(`${id}-name`));
  }, [id]);

  return (
    <Grid item xs={12} sm={6}>
      <Card variant="outlined" sx={{ borderRadius: 0 }}>
        <Link href={`/home/scene/${id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={preview ?? identicon}
            />
            <CardContent sx={{ p: 2, borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              <Typography>🚧 {name ?? id}</Typography>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
