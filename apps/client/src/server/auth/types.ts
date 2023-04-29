import { z } from "zod";

export enum AuthMethod {
  Ethereum = "Ethereum",
}

export const EthereumAuthSchema = z.object({
  method: z.literal(AuthMethod.Ethereum),
  message: z.string(),
  signature: z.string(),
});

export const AuthSchema = EthereumAuthSchema;

export type AuthData = z.infer<typeof AuthSchema>;
