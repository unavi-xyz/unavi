import { ReactChild } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
} from "@mui/material";
import Link from "next/link";

interface Props {
  href: string;
  image?: string;
  children?: ReactChild | ReactChild[];
}

export default function BasicCard({ href, image, children }: Props) {
  return (
    <Grid item xs={12} sm={6}>
      <Card variant="outlined" sx={{ borderRadius: 0, height: "200px" }}>
        <Link href={href} passHref>
          <CardActionArea>
            <CardMedia component="img" height="140px" image={image} />
            <CardContent sx={{ p: 2, borderTop: "1px solid rgba(0,0,0,0.12)" }}>
              {children}
            </CardContent>
          </CardActionArea>
        </Link>
      </Card>
    </Grid>
  );
}
