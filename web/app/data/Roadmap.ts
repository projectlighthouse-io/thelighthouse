import type { RoadmapColumn } from '@/types/Content'

// Static until the rust content endpoints exist — see docs/rebuild.md phase 2.
// Copied from resources/js/pages/Roadmap.vue in the laravel repo.

export const columns: RoadmapColumn[] = [
  {
    label: 'Ideas',
    items: [
      {
        title: 'Protocol Engineering',
        description: 'Design wire protocols from scratch. Text-based (Redis RESP), binary framing, TLV encoding, flow control.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Distributed System Design',
        description: 'Raft, replicated KV, sharded KV, 2PC, SWIM, CRDTs. Each chapter is its own project.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Database Internals',
        description: 'B-trees, WAL, MVCC, buffer pools, query execution. How databases actually work.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Observability & Performance',
        description: 'Prometheus, OpenTelemetry, eBPF, flame graphs, distributed tracing.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Orchestrating Containers',
        description: 'Kubernetes internals, Firecracker microVMs. How orchestration works, not how to use kubectl.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Advanced Algorithms',
        description: 'Segment trees, tries, skip lists, bloom filters, consistent hashing. Algorithms in real systems.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Firecracker microVM',
        description: 'Replace Docker containers with Firecracker microVMs for stronger isolation and faster boot times.',
        tags: ['labs', 'platform'],
        hot: false,
      },
      {
        title: 'Collaborative terminals',
        description: 'Shared terminal sessions for pair programming and mentoring.',
        tags: ['labs'],
        hot: false,
      },
      {
        title: 'Team mode',
        description: 'Invite teams, track collective progress, and manage lab access for groups.',
        tags: ['platform'],
        hot: false,
      },
    ],
  },
  {
    label: 'Next',
    items: [
      {
        title: 'Build Your Own Docker project',
        description: '8-task project: namespaces, cgroups, chroot, networking. Build a container runtime from scratch.',
        tags: ['labs'],
        hot: true,
      },
      {
        title: 'Build Your Own Redis project',
        description: 'Local project with luxctl. RESP protocol, key-value store, persistence. Go and Rust.',
        tags: ['labs'],
        hot: true,
      },
      {
        title: 'Container Internals book',
        description: 'Namespaces, cgroups, OverlayFS, OCI images, container runtimes, networking, security. 25-35 lessons.',
        tags: ['books'],
        hot: true,
      },
      {
        title: 'DSA — unlock trees, graphs, heaps, hashing, bits',
        description: 'Finish reviewing and publish remaining DSA chapters.',
        tags: ['books'],
        hot: true,
      },
    ],
  },
  {
    label: 'Forging',
    items: [
      {
        title: 'Networking Fundamentals — Bangla translation',
        description: 'Bangla version landing lesson by lesson once each translation passes review. /bn falls back to English while unpublished.',
        tags: ['books'],
        hot: true,
      },
      {
        title: 'OS Fundamentals — Bangla translation',
        description: 'Bangla version landing lesson by lesson once each translation passes review. /bn falls back to English while unpublished.',
        tags: ['books'],
        hot: true,
      },
      {
        title: 'Rust from Zero — Bangla translation',
        description: 'Bangla version landing lesson by lesson once each translation passes review. /bn falls back to English while unpublished.',
        tags: ['books'],
        hot: true,
      },
      {
        title: 'DSA Fundamentals — Bangla translation',
        description: 'Bangla version landing lesson by lesson once each translation passes review. /bn falls back to English while unpublished.',
        tags: ['books'],
        hot: true,
      },
      {
        title: 'Shipping Go Web Services — Bangla translation',
        description: 'Bangla version landing lesson by lesson once each translation passes review. /bn falls back to English while unpublished.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Crack the Interview — early lessons',
        description: 'Systems-flavored interview prep: behavioural, system design, and DSA framing. Publishing as chapters review.',
        tags: ['books'],
        hot: true,
      },
    ],
  },
  {
    label: 'Shipped',
    items: [
      {
        title: 'Rust 101s — focused mini-lessons',
        description: 'Short, focused Rust drills that complement Rust from Zero. Published one drill at a time.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Lab page redesign — problem brief',
        description: 'New tabbed brief layout on every terminal/lab page. Sandbox, DSA, systems, and troubleshooting labs all share a clean reader-style left pane with the terminal sliding in from the right.',
        tags: ['labs', 'platform'],
        hot: false,
      },
      {
        title: 'Project Challenge & Show redesign',
        description: 'New Challenge and Show pages for projects with refreshed luxctl block, hint UX, and Echo deduping.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'DSA index page at /dsa',
        description: 'Dedicated landing page for all DSA labs, grouped by topic and difficulty.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'Paywall rebuild + BD checkout',
        description: 'Refreshed Pro membership block with dynamic early-member count, wired-up Stripe checkout, and a card-free Bangladesh deeplink.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'BDT vault — card-free Bangladesh payments',
        description: 'Bangladesh-only payment flow with bKash/Nagad/bank options. Admin payment-request viewer with tabbed email templates.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'Reader rebuild — ReaderShell composable',
        description: 'All lesson pages now share a single composable ReaderShell with cleaner grid, sub-bar, TOC, and aside. Reusable across blog, lessons, and lab briefs.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'Go Fundamentals — Bangla translation live',
        description: 'Go Fundamentals Bangla lessons published. Per-locale publishing with English fallback on /bn.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Go Intermediate — Bangla translation live',
        description: 'Go Intermediate Bangla lessons published. Per-locale publishing with English fallback on /bn.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'C Programming — Bangla translation live',
        description: 'C Programming Bangla lessons published. Per-locale publishing with English fallback on /bn.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Public profile rework',
        description: 'Rebuilt /users/@me with fixed 301 cache, redesigned profile cards, and admin user viewer.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'v1.0 launch',
        description: 'Platform launched with 8 books, 51 DSA labs, 10 CLI challenges, 3 build-from-scratch projects, remote terminals, and Forge code execution.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'Go Fundamentals book',
        description: 'Variables, types, structs, pointers, functions, testing, error handling, interfaces.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Go Intermediate book',
        description: 'GMP scheduler, channels, sync primitives, context, generics, profiling.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Shipping Go Web Services book',
        description: 'HTTP server from scratch, PostgreSQL, middleware, auth, dashboards, deployment.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'DSA Fundamentals book',
        description: 'Arrays, linked lists, stacks, queues, recursion, sorting, binary search — with 51 hands-on labs.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'OS Fundamentals book',
        description: 'Processes, fork/exec, process states, threads vs processes, memory management.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Networking Fundamentals book',
        description: 'TCP/IP, sockets, DNS, HTTP internals, and packet analysis from first principles.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'C Programming book',
        description: 'Pointers, memory, structs, the compilation model, and building real CLI tools.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'Rust from Zero book',
        description: 'Ownership, borrowing, lifetimes, traits. Building a CLI password manager.',
        tags: ['books'],
        hot: false,
      },
      {
        title: 'CLI challenges — grep, sed, awk, jq, find, pipes, regex, shell scripts, sort/uniq',
        description: '10 tool-specific challenges with 8 tasks each, running on remote lab containers.',
        tags: ['labs'],
        hot: false,
      },
      {
        title: 'Build Your Own HTTP Server',
        description: '10-task project: parse requests, route, respond, handle headers, concurrent connections.',
        tags: ['labs'],
        hot: false,
      },
      {
        title: 'SeaChart: DNS Resolver',
        description: 'Build a DNS resolver from scratch with recursive resolution and caching.',
        tags: ['labs'],
        hot: false,
      },
      {
        title: '1 Billion Row Challenge',
        description: 'Performance-focused local lab parsing 1B rows in Go, Rust, or C.',
        tags: ['labs'],
        hot: false,
      },
      {
        title: 'Forge — DSA code execution',
        description: 'In-browser code editor with sandboxed execution for DSA labs. No VM needed.',
        tags: ['labs', 'platform'],
        hot: false,
      },
      {
        title: 'Projects & challenges split',
        description: 'Tabbed view separating build-from-scratch projects from tool-specific challenges.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'Code-group tabbed code blocks',
        description: 'Multi-language code examples with C/Go/Rust tabs in lesson content.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'Remote VM terminals',
        description: 'Browser-based SSH terminals powered by Sentinel for cloud lab environments.',
        tags: ['labs'],
        hot: false,
      },
      {
        title: 'Subscription & payments',
        description: 'Stripe-powered subscriptions with gift codes and coupon support.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'Leaderboard & XP system',
        description: 'Gamified learning with points, leaderboard rankings, and achievement tracking.',
        tags: ['platform'],
        hot: false,
      },
      {
        title: 'luxctl Windows support',
        description: 'Install script now detects Windows (Git Bash/MSYS2) and downloads pre-built .exe.',
        tags: ['labs'],
        hot: false,
      },
    ],
  },
]
