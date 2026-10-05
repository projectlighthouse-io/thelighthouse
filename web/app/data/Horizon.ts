/**
 * What comes after the desk: the books planned but not yet being written, and
 * the luxctl projects planned beside them. For the home page's horizon list.
 *
 * The books are the plan's #4–#15, in its order and grouped by the bundle each
 * is meant for; #1–#3 are the published foundations. The projects are the
 * headline builds from docs/lighthouse-roadmap.md that are not on the shelf
 * yet. Titles and one line each — nothing here claims progress, so nothing
 * here has to be kept honest beyond the list itself.
 */

export interface PlannedBook {
  title: string
  /** The bundle the plan puts it in. */
  bundle: string
}

export interface PlannedProject {
  title: string
  /** What you build, in a line. */
  line: string
}

export const plannedBooks: PlannedBook[] = [
  { title: 'Build Your Own Container', bundle: 'Containers & Orchestration' },
  { title: 'Containerizing Your App', bundle: 'Containers & Orchestration' },
  { title: 'Orchestration Without Kubernetes', bundle: 'Containers & Orchestration' },
  { title: 'Orchestrating Containers', bundle: 'Containers & Orchestration' },
  { title: 'Kubernetes Networking', bundle: 'Platform Engineering' },
  { title: 'Programming Kubernetes', bundle: 'Platform Engineering' },
  { title: 'Operating Kubernetes at Scale', bundle: 'Platform Engineering' },
  { title: 'eBPF & Kernel Observability', bundle: 'Platform Engineering' },
  { title: 'Build Your Own Prometheus', bundle: 'Platform Engineering' },
  { title: 'System Design', bundle: 'Architect' },
  { title: 'Distributed System Design', bundle: 'Architect' },
  { title: 'Database Internals', bundle: 'Architect' },
]

export const plannedProjects: PlannedProject[] = [
  { title: 'Build Your Own Redis', line: 'The RESP protocol, a key-value store and persistence, in Go or Rust.' },
  { title: 'Build Your Own Git', line: 'Objects, refs and packfiles — the content-addressed store underneath.' },
  { title: 'Build Your Own Orchestrator', line: 'An agent, a scheduler and a state store, without Kubernetes.' },
  { title: 'Build Your Own Raft', line: 'Leader election and log replication behind a replicated key-value store.' },
  { title: 'Build Your Own Operator', line: 'A custom resource and the reconcile loop that keeps it true.' },
  { title: 'Build Your Own Kubernetes', line: 'An API server, a scheduler and a kubelet, assembled from your own parts.' },
]
