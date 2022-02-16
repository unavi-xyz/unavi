import { useEffect, useState } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Skeleton,
  Typography,
} from "@mui/material";
import Link from "next/link";
import { useIdenticon } from "ui";
import { useWorld } from "ceramic";

interface Props {
  id: string;
}

export default function WorldCard({ id }: Props) {
  const { world } = useWorld(id);
  const identicon = useIdenticon(id);

  const [name, setName] = useState("");

  useEffect(() => {
    if (!world) return;
    if (world.name?.length > 0) setName(world.name);
    else setName(id);
  }, [id, world]);

  return (
    <Grid item xs={12} sm={6} md={4}>
      <Card variant="outlined" sx={{ borderRadius: 0 }}>
        <Link href={`/home/world/${id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={world?.image ?? identicon}
            />
            <CardContent style={{ borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              {name ? (
                <Typography style={{ wordBreak: "break-word" }}>
                  🌏 {name}
                </Typography>
              ) : (
                <Skeleton variant="text" />
              )}
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
