import { z } from "zod";

export enum AuthMethod {
  Ethereum = "ethereum",
}

export const EthereumAuthSchema = z.object({
  message: z.string(),
  method: z.literal(AuthMethod.Ethereum),
  signature: z.string(),
});

export const AuthSchema = EthereumAuthSchema;

export type AuthData = z.infer<typeof AuthSchema>;
