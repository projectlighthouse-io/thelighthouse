import type { Manuscript } from '@/types/Content'

/**
 * What is on the desk: the books being written now, for the home page's desk
 * and the roadmap.
 *
 * SEED DATA. The titles and descriptions are the real horizon list; the
 * chapter outlines, written counts, excerpts and edit dates are the redesign's
 * placeholder copy, kept so the block renders. Replace each with the draft's
 * real outline and a real line from it before this ships — an excerpt is
 * presented as a quote from the manuscript.
 */
export const manuscripts: Manuscript[] = [
  {
    title: 'Containerizing Your App',
    status: 'next',
    description:
      'Take Logline (Go) and Rune (Rust) from earlier books and ship them. Dockerfiles, multi-stage builds, registries, health checks — the bridge between "runs on my machine" and "runs anywhere".',
    chapters: [
      'What a process is allowed to see',
      'Namespaces, one at a time',
      'cgroups and the memory limit',
      'The layered filesystem',
      'Building an image by hand',
      'Networking a container',
      'What docker run actually does',
      'Kubernetes, from the pod inward',
    ],
    written: 5,
    excerpt: {
      from: 'chapter 06',
      text: 'A container has no network. It has a namespace with a loopback in it, and everything after that is something you plugged in.',
    },
    editedAt: '2026-10-02',
  },
  {
    title: 'Distributed System Design',
    status: 'drafting',
    description:
      'Consensus, replication, partitioning, CRDTs — each chapter a build-from-scratch project. Implement Raft, build a replicated KV store, write a SWIM-based membership service, ship a CRDT that merges without conflicts.',
    chapters: [
      'Two machines and a clock',
      'Replication',
      'Failure detection',
      'Leader election',
      'Consensus',
      'Partitions, and what you give up',
      'Exactly-once is a lie',
      'Testing a system you cannot see',
    ],
    written: 2,
    excerpt: {
      from: 'chapter 02',
      text: 'The follower is not behind. It is simply living a few hundred milliseconds in the past, and every bug in this chapter comes from forgetting that.',
    },
    editedAt: '2026-09-25',
  },
  {
    title: 'Protocol Engineering',
    status: 'outlined',
    description:
      'Design and implement wire protocols from scratch. Start text-based (Redis RESP), then binary framing — length-prefixed, TLV, versioning. Capstone: a custom protocol controlling real hardware over TCP.',
    chapters: [
      'Bytes on a wire',
      'Framing',
      'Versioning from day one',
      'Encoding numbers, strings, and nothing',
      'Backwards compatibility',
      'When both ends ship separately',
    ],
    written: 0,
    excerpt: {
      from: 'the outline',
      text: 'Every protocol is a contract signed by two programs that will never be deployed at the same moment.',
    },
    editedAt: '2026-09-13',
  },
]
