import { z } from "zod";

export const proposalStatusSchema = z.enum(["active", "passed", "rejected"]);

export const proposalMetadataSchema = z.object({
  title: z
    .string()
    .min(1, "Title is required")
    .min(3, "Title must be at least 3 characters")
    .max(200, "Title must be less than 200 characters"),
  body: z
    .string()
    .min(1, "Proposal description is required")
    .min(10, "Description must be at least 10 characters")
    .max(10000, "Description must be less than 10,000 characters"),
  deadline: z
    .date()
    .refine(
      (date: Date) => date > new Date(),
      "Deadline must be in the future"
    ),
  voters: z
    .array(z.string().regex(/^0x[a-fA-F0-9]{64}$/, "Invalid ZK address format"))
    .min(1, "At least one voter is required")
    .max(1000, "Maximum 1000 voters allowed"),
});

export const proposalSchema = z.object({
  id: z.number().int().positive(),
  metadata: proposalMetadataSchema,
  deadline: z.date(),
  createdAt: z.date(),
  author: z.string().regex(/^0x[a-fA-F0-9]{64}$/, "Invalid author address"),
  for: z.number().int().min(0),
  against: z.number().int().min(0),
  status: proposalStatusSchema,
});

// Address validation schema (reusable)
export const zkAddressSchema = z
  .string()
  .min(1, "Address is required")
  .regex(/^0x[a-fA-F0-9]{64}$/, "Invalid ZK address format");

// Exported TypeScript types
export type ProposalStatus = z.infer<typeof proposalStatusSchema>;
export type ProposalMetadata = z.infer<typeof proposalMetadataSchema>;
export type Proposal = z.infer<typeof proposalSchema>;
export type ZkAddress = z.infer<typeof zkAddressSchema>;
