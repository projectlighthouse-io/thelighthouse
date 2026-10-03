export interface Book {
  slug: string
  title: string
  description: string
  thumbnailUrl: string
  /** Which tracks the book is on, and where it falls in each — `{go: 4}`. The
   *  book's own yaml says this; nothing here infers it. Empty means untracked,
   *  which is not the same as last. */
  tracks: Record<string, number>
  pages: number
  /**
   * A price tag, or null when the book is free. Never a number: the money
   * arithmetic stays in minor units on the rust side, and a float here is how
   * a display value ends up being charged.
   */
  price: string | null
  /** Where "start reading" goes. Null for a book with no published lessons. */
  firstLesson?: string | null
  /** The book page's slideshow, in reading order. Empty for a book with none.
   *  Optional because the listing endpoint does not send it. */
  images?: string[]
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
  /** A challenge has no companion book: the tasks are the whole thing. Which
   *  of the two tabs a project belongs in, decided in `project.yaml` rather
   *  than by which list it was typed into. */
  isChallenge: boolean
}

/** A task as a project's contents list renders it. */
export interface ProjectTask {
  slug: string
  title: string
  sortOrder: number
  points: number
  isFree: boolean
}

export interface ProjectFeature {
  title: string
  description: string
  icon: string
}

/** One project's own page: everything the listing has, plus the pitch and the
 *  tasks by name. */
export interface ProjectPage extends Project {
  headline: string
  longDescription: string
  difficulty: string
  unlockMode: 'open' | 'sequential'
  relatedBook: string | null
  features: ProjectFeature[]
  tasks: ProjectTask[]
}

/** One task's own page: the brief, and how to get around. */
export interface TaskPage {
  slug: string
  title: string
  sortOrder: number
  points: number
  isFree: boolean
  html: string
  position: number
  total: number
  project: { slug: string, name: string }
  previous: { slug: string, title: string } | null
  next: { slug: string, title: string } | null
}

/** Where a reader stands on one task, as the progress poll reports it. */
export interface TaskProgress {
  slug: string
  status:
    | 'challenge_awaits'
    | 'challenged'
    | 'challenge_completed'
    | 'challenge_failed'
    | 'challenge_abandoned'
  attempts: number
  points_earned: number
  is_locked: boolean
  is_paid: boolean
  started_at: string | null
  completed_at: string | null
}

/** What one poll returns. */
export interface ProjectProgress {
  run: number
  completed: number
  total: number
  points_earned: number
  tasks: TaskProgress[]
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
  /** A plain-text excerpt of the body. Never html — see `SafeMarkdown`. */
  description: string
  publishedAt: string
  /** The article's one category, as a list so the existing card markup that
   *  renders tags needs no change. */
  tags: string[]
  readMinutes: number
  author: string
  authorUsername: string | null
  /** Only on the author's own listing — `/blog?author=<them>` read by them.
   *  `null` means live; a date means it is not being served. */
  takenDownAt?: string | null
  /** Why, shown to its author and to nobody else. */
  takenDownReason?: string | null
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

/**
 * A lesson's contents entry, which carries one thing a syntax page's does not.
 *
 * The api sends every heading to everybody and marks the ones this reader
 * cannot reach — so a locked lesson still shows what is in it, and only the
 * anchors change. Its own type rather than an optional field on `TocItem`:
 * a syntax page has no paywall, and `locked?: boolean` there would be a
 * question nothing can answer.
 */
export interface LessonTocItem extends TocItem {
  locked: boolean
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
  updatedAt: string
  /** Sanitised on the server by `SafeMarkdown.renderArticle`. This is the only
   *  field in the app that is safe to put in `v-html`, and the unsanitised
   *  markdown is deliberately not sent alongside it. */
  html: string
}

/** What `/_api/blog` answers: a page of cards, plus the count. */
export interface BlogListResponse {
  items: BlogPost[]
  page: number
  per_page: number
  total: number
}

export interface LessonResponse {
  book: { slug: string, title: string, thumbnailUrl: string }
  lesson: { slug: string, title: string, description: string, locked: boolean }
  html: string
  toc: LessonTocItem[]
  readMinutes: number
  remainingSections: number
  position: number
  total: number
  percent: number
  previous: { slug: string, title: string } | null
  next: { slug: string, title: string } | null
  /** The language `html` is in. */
  locale: string
  /** Every language this lesson is written in, for the switcher. */
  locales: string[]
}
