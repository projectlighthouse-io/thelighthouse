export interface Book {
  slug: string
  title: string
  description: string
  thumbnailUrl: string
  pages: number
  price: string | null
  inProgress: boolean
}

export interface HeroStat {
  value: string
  label: string
}

export interface Project {
  slug: string
  name: string
  shortDescription: string
  tasksCount: number
}

export interface HorizonBook {
  title: string
  description: string
}

export interface Testimonial {
  quote: string
}

export interface Faq {
  question: string
  answer: string
}

export interface Chapter {
  id: number
  title: string
}

export interface LessonSummary {
  slug: string
  title: string
  description: string
  chapterId: number
  locked: boolean
}

export interface Curriculum {
  chapters: Chapter[]
  lessons: LessonSummary[]
}

export interface BlogPost {
  slug: string
  title: string
  description: string
  publishedAt: string
  tags: string[]
  readMinutes: number
}

export interface SyntaxLanguage {
  slug: string
  name: string
  description: string
}

export interface RoadmapItem {
  title: string
  description: string
  tags: string[]
  hot: boolean
}

export interface RoadmapColumn {
  label: string
  headerClass: string
  dotClass: string
  cardClass: string
  items: RoadmapItem[]
}

export interface ChangelogEntry {
  date: string
  title: string
  description: string
  tag: string
}

export interface TocItem {
  id: string
  text: string
}

export interface SyntaxResponse {
  slug: string
  name: string
  description: string
  html: string
  toc: TocItem[]
  readMinutes: number
}

export interface BlogPostResponse extends BlogPost {
  html: string
}

export interface LessonResponse {
  book: { slug: string, title: string, thumbnailUrl: string }
  lesson: { slug: string, title: string, description: string, locked: boolean }
  html: string
  toc: TocItem[]
  readMinutes: number
  remainingSections: number
  position: number
  total: number
  percent: number
  previous: { slug: string, title: string } | null
  next: { slug: string, title: string } | null
}
