import { useContext } from "react";
import { Button } from "@mui/material";
import Link from "next/link";

import { MatrixContext } from "matrix";

export default function LoginButton() {
  const { loggedIn, logout } = useContext(MatrixContext);

  return (
    <div>
      {loggedIn ? (
        <Link href="/home/login" passHref>
          <Button variant="outlined" onClick={logout}>
            Logout
          </Button>
        </Link>
      ) : (
        <Link href="/home/login" passHref>
          <Button variant="contained">Login</Button>
        </Link>
      )}
    </div>
  );
}
