import { ReactChild } from "react";
import { Box, Modal, Stack, Typography } from "@mui/material";

interface Props {
  open: boolean;
  title?: string;
  handleClose: () => void;
  children: ReactChild | ReactChild[];
}

export default function BasicModal({
  open,
  handleClose,
  title = "",
  children,
}: Props) {
  return (
    <Modal open={open} onClose={handleClose}>
      <Box
        sx={{
          position: "absolute",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          maxWidth: 500,
          width: "100%",
          bgcolor: "background.paper",
          border: "2px solid #000",
          boxShadow: 24,
          borderRadius: 2,
          p: 4,
        }}
      >
        <Stack spacing={3}>
          <Typography variant="h5">{title}</Typography>
          {children}
        </Stack>
      </Box>
    </Modal>
  );
}
