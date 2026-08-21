import type { SyntaxLanguage } from '@/types/Content'

// Metadata only. The markdown lives in server/data/SyntaxBodies.ts so a page
// cannot pull 100kb of it into the client with an innocent-looking import.

export const languages: SyntaxLanguage[] = [
  {
    slug: "go",
    name: "Go",
    description: "Statically typed, compiled, concurrent.",
  },
  {
    slug: "rust",
    name: "Rust",
    description: "Memory-safe systems programming.",
  },
  {
    slug: "c",
    name: "C",
    description: "The foundation of modern computing.",
  },
]
