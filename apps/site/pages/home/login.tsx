import Head from "next/head";
import { LoginPage } from "ui";

export default function Login() {
  return (
    <div>
      <Head>
        <title>The Wired - Home</title>
      </Head>
      <LoginPage finishedHref="/home" registerHref="/home/register" />;
    </div>
  );
}
