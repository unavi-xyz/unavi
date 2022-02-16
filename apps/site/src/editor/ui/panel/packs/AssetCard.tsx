import { useEffect, useState } from "react";
import {
  Card,
  CardActionArea,
  CardContent,
  Grid,
  Typography,
} from "@mui/material";
import { ASSETS, ASSET_NAMES } from "3d";

import { useStore } from "../../../hooks/useStore";

interface Props {
  name: ASSET_NAMES;
}

export default function AssetCard({ name }: Props) {
  const objects = useStore((state) => state.objects);
  const newObject = useStore((state) => state.newObject);

  const [count, setCount] = useState(0);

  const limit = ASSETS[name].limit;

  function handleClick() {
    newObject(name);
  }

  useEffect(() => {
    const found = Object.values(objects).filter(
      (item) => item.instance.type === name
    );

    setCount(found.length);
  }, [name, objects]);

  return (
    <Grid item xs sx={{ minWidth: "120px" }}>
      <Card variant="outlined">
        <CardActionArea onClick={handleClick}>
          <CardContent>
            <Grid container>
              <Grid item xs={3}></Grid>

              <Grid item xs={6} container justifyContent="center">
                <Typography>{name}</Typography>
              </Grid>

              <Grid item xs={3} container justifyContent="flex-end">
                {limit > -1 && (
                  <Typography>
                    {count}/{limit}
                  </Typography>
                )}
              </Grid>
            </Grid>
          </CardContent>
        </CardActionArea>
      </Card>
    </Grid>
  );
}
