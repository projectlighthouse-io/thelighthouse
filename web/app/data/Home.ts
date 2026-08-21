import type { HeroStat, HorizonBook } from '@/types/Content'

// Static until the rust content endpoints exist — see docs/rebuild.md phase 2.

export const heroStats: HeroStat[] = [
  { value: '10', label: 'books' },
  { value: '3', label: 'projects' },
  { value: '9', label: 'challenges' },
  { value: '~1,800', label: 'pages worth over 350 lessons' },
]

export const horizonBooks: HorizonBook[] = [
  {
    title: 'Containerizing Your App',
    description:
      'Take Logline (Go) and Rune (Rust) from earlier books and ship them. Dockerfiles, multi-stage builds, registries, health checks — the bridge between "runs on my machine" and "runs anywhere".',
  },
  {
    title: 'Advanced Containers',
    description:
      'What actually happens when you run docker build. Namespaces, cgroups, overlay filesystems, OCI specs, BuildKit internals, rootless runtimes. After this, Docker stops being magic.',
  },
  {
    title: 'Protocol Engineering',
    description:
      'Design and implement wire protocols from scratch. Start text-based (Redis RESP), then binary framing — length-prefixed, TLV, versioning. Capstone: a custom protocol controlling real hardware over TCP.',
  },
  {
    title: 'Distributed System Design',
    description:
      'Consensus, replication, partitioning, CRDTs — each chapter a build-from-scratch project. Implement Raft, build a replicated KV store, write a SWIM-based membership service, ship a CRDT that merges without conflicts.',
  },
  {
    title: 'Database Internals',
    description:
      'How databases actually work under the hood. B-trees flushed to disk, write-ahead logs, MVCC, buffer pools, query executors, LSM trees vs hash indexes. Where the OS and DSA books become a storage engine.',
  },
  {
    title: 'Observability & Performance',
    description:
      'You built the systems — now see inside them. Prometheus metrics, OpenTelemetry traces, eBPF syscall probes, flame graphs, structured logs. Profile GC pauses and lock contention until the regression is obvious.',
  },
  {
    title: 'Orchestrating Containers',
    description:
      'Not "how to use kubectl" — how orchestration actually works. Kubelet pod lifecycles, CNI networking, scheduler placement, Firecracker microVMs. Write your own mini-scheduler from scratch.',
  },
  {
    title: 'Advanced Algorithms',
    description:
      'Algorithms that show up in real systems, not interview prep. Segment trees for time-series, tries for routing, skip lists in Redis, bloom filters in LSM trees, consistent hashing for sharding, A* for pathfinding.',
  },
]
