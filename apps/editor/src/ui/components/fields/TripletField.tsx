import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { Grid, Typography } from "@mui/material";
import { Triplet } from "@react-three/cannon";

import NumberField from "./NumberField";

interface Props {
  title: string;
  step?: number;
  value: Triplet;
  setValue: Dispatch<SetStateAction<Triplet>>;
}

export default function TripletField({ title, step, value, setValue }: Props) {
  const [x, setX] = useState(value[0]);
  const [y, setY] = useState(value[1]);
  const [z, setZ] = useState(value[2]);

  useEffect(() => {
    setValue([x, y, z]);
  }, [setValue, x, y, z]);

  useEffect(() => {
    setX(value[0]);
    setY(value[1]);
    setZ(value[2]);
  }, [value]);

  return (
    <Grid container direction="row" alignItems="center" columnSpacing={2}>
      <Grid item xs={3}>
        <Typography>{title}</Typography>
      </Grid>
      <Grid item xs={3}>
        <NumberField step={step} value={x} setValue={setX} />
      </Grid>
      <Grid item xs={3}>
        <NumberField step={step} value={y} setValue={setY} />
      </Grid>
      <Grid item xs={3}>
        <NumberField step={step} value={z} setValue={setZ} />
      </Grid>
    </Grid>
  );
}
