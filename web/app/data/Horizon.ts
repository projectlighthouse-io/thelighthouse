/**
 * What comes after the desk: the books planned but not yet being written,
 * grouped into the tracks they are meant for, and the luxctl projects planned
 * beside them. For the home page's "And after that" section.
 *
 * Titles and a line each — nothing here claims progress, so nothing here has
 * to be kept honest beyond the list itself. Every count on the page is derived
 * from these arrays; add a book or a project here and the numbers follow.
 */

export interface PlannedTrack {
  name: string
  /** In reading order. */
  books: string[]
}

export interface PlannedProject {
  title: string
  /** What you build, in a line. */
  blurb: string
}

export const plannedTracks: PlannedTrack[] = [
  {
    name: 'Containers & Orchestration',
    books: [
      'Build Your Own Container',
      'Containerizing Your App',
      'Orchestration Without Kubernetes',
      'Orchestrating Containers',
    ],
  },
  {
    name: 'Platform Engineering',
    books: [
      'Kubernetes Networking',
      'Programming Kubernetes',
      'Operating Kubernetes at Scale',
      'eBPF & Kernel Observability',
      'Build Your Own Prometheus',
    ],
  },
  {
    name: 'Architect',
    books: [
      'System Design',
      'Distributed System Design',
      'Database Internals',
    ],
  },
]

export const plannedProjects: PlannedProject[] = [
  { title: 'Build Your Own Redis', blurb: 'The RESP protocol, a key-value store and persistence, in Go or Rust.' },
  { title: 'Build Your Own Git', blurb: 'Objects, refs and packfiles: the content-addressed store underneath.' },
  { title: 'Build Your Own Orchestrator', blurb: 'An agent, a scheduler and a state store, without Kubernetes.' },
  { title: 'Build Your Own Raft', blurb: 'Leader election and log replication behind a replicated key-value store.' },
  { title: 'Build Your Own Operator', blurb: 'A custom resource and the reconcile loop that keeps it true.' },
  { title: 'Build Your Own Kubernetes', blurb: 'An API server, a scheduler and a kubelet, assembled from your own parts.' },
]
