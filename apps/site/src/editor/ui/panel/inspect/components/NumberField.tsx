import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { Grid, TextField, Typography } from "@mui/material";
import { ASSETS, ASSET_NAMES, PARAM_NAMES } from "3d";
import { degToRad, radToDeg } from "three/src/math/MathUtils";

interface Props {
  title: string;
  type: PARAM_NAMES;
  params: typeof ASSETS[ASSET_NAMES]["params"];
  setParams: Dispatch<SetStateAction<typeof ASSETS[ASSET_NAMES]["params"]>>;
  variant?: "radians";
  step?: number;
  min?: number;
  max?: number;
}

export default function NumberField({
  title,
  type,
  params,
  setParams,
  variant,
  step = 0.1,
  max,
  min,
}: Props) {
  const single = typeof params[type] === "number";

  const [displayed, setDisplayed] = useState<(string | number)[]>([]);

  useEffect(() => {
    const value: number[] = single ? [params[type]] : params[type];

    const rounded = value.map((item) => {
      const converted = variant === "radians" ? radToDeg(item) : item;
      return Math.round(converted * 100) / 100;
    });

    setDisplayed(rounded);
  }, [params, single, type, variant]);

  return (
    <Grid container direction="row" alignItems="center" columnSpacing={2}>
      <Grid item xs={3}>
        <Typography>{title}</Typography>
      </Grid>

      {displayed.map((item, i) => {
        function handleChange(e) {
          if (!e.target.value) {
            setDisplayed((prev) => {
              const newDisplayed = [...prev];
              newDisplayed[i] = "";
              return newDisplayed;
            });
            return;
          }

          setParams((prev) => {
            const newParams = { ...prev };
            const converted =
              variant === "radians"
                ? degToRad(Number(e.target.value))
                : Number(e.target.value);

            if (single) newParams[type] = converted;
            else newParams[type][i] = converted;
            return newParams;
          });
        }

        return (
          <Grid key={i} item xs>
            <TextField
              value={item}
              onChange={handleChange}
              type="number"
              variant="standard"
              fullWidth
              inputProps={{ step, max, min }}
            />
          </Grid>
        );
      })}
    </Grid>
  );
}
