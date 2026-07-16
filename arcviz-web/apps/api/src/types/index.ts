import { z } from 'zod'

export const registerSchema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
  name: z.string().optional(),
})

export const loginSchema = z.object({
  email: z.string().email(),
  password: z.string(),
})

export const createProjectSchema = z.object({
  name: z.string().min(1),
  description: z.string().optional(),
  visibility: z.enum(['PRIVATE', 'PUBLIC', 'ORGANIZATION']).default('PRIVATE'),
})

export const updateProjectSchema = z.object({
  name: z.string().min(1).optional(),
  description: z.string().optional(),
  visibility: z.enum(['PRIVATE', 'PUBLIC', 'ORGANIZATION']).optional(),
})

export const createFileSchema = z.object({
  name: z.string().min(1),
  path: z.string().min(1),
  content: z.string(),
  language: z.string().default('arclang'),
})

export const updateFileSchema = z.object({
  name: z.string().min(1).optional(),
  content: z.string().optional(),
})

export const compileSchema = z.object({
  code: z.string(),
  fileId: z.string().optional(),
})

export type RegisterInput = z.infer<typeof registerSchema>
export type LoginInput = z.infer<typeof loginSchema>
export type CreateProjectInput = z.infer<typeof createProjectSchema>
export type UpdateProjectInput = z.infer<typeof updateProjectSchema>
export type CreateFileInput = z.infer<typeof createFileSchema>
export type UpdateFileInput = z.infer<typeof updateFileSchema>
export type CompileInput = z.infer<typeof compileSchema>
