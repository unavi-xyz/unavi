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
import { useWorld } from "ceramic";

interface Props {
  id: string;
}

export default function WorldCard({ id }: Props) {
  const world = useWorld(id);
  const identicon = useIdenticon(id);

  return (
    <Grid item xs={12} sm={6} md={4}>
      <Card variant="outlined">
        <Link href={`/home/world/${id}`} passHref>
          <CardActionArea>
            <CardMedia
              component="img"
              height="140px"
              image={world?.image ?? identicon}
            />
            <CardContent style={{ borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              <Typography>{world?.name}</Typography>
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
