import type { Project } from '@/types/Content'

// Static until the rust content endpoints exist — see docs/rebuild.md phase 2.

export const projects: Project[] = [
  {
    slug: 'build-your-own-http-server',
    name: 'Build Your Own HTTP Server',
    shortDescription:
      'Build an HTTP server from raw TCP sockets. No frameworks, no libraries — just you and the protocol specification. Understand exactly what happens when browsers talk to servers.',
    tasksCount: 9,
  },
  {
    slug: 'dns-resolver',
    name: 'SeaChart: Build Your Own DNS Resolver',
    shortDescription:
      'Understand DNS from the ground up by building a recursive resolver. Parse binary protocols, query root servers, follow delegation chains, and implement caching.',
    tasksCount: 8,
  },
  {
    slug: '1-billion-row-challenge',
    name: '1 Billion Row Challenge',
    shortDescription:
      "Process 1 billion temperature measurements as fast as possible. Start with a working solution, then systematically optimize — I/O, parallelism, parsing, data structures — until you're competing with the best.",
    tasksCount: 7,
  },
]

export const challenges: Project[] = [
  {
    slug: 'finding-files-with-find',
    name: 'Finding Files with find',
    shortDescription:
      'Disk alerts are firing and orphan files are everywhere. Locate files by pattern, age, and size, then clean up in bulk — right tools, tree node, find and xargs.',
    tasksCount: 6,
  },
  {
    slug: 'unix-pipelines',
    name: 'Unix Pipelines',
    shortDescription:
      "Single commands aren't enough. Chain grep, sed, awk, sort, and uniq into multi-stage pipelines that answer real questions about real data.",
    tasksCount: 6,
  },
  {
    slug: 'stream-editing-with-sed',
    name: 'Stream Editing with sed',
    shortDescription:
      'A deploy went wrong and you are on-call. Fix broken configs, extract errors from logs, and rewrite files in place without opening an editor.',
    tasksCount: 5,
  },
  {
    slug: 'shell-scripting-basics',
    name: 'Shell Scripting Basics',
    shortDescription:
      'You keep typing the same commands over and over. Automate your on-call workflow with scripts that take arguments, branch, loop, and fail loudly.',
    tasksCount: 6,
  },
  {
    slug: 'log-analysis-with-awk',
    name: 'Log Analysis with awk',
    shortDescription:
      'A deploy went wrong and you are on-call. Extract columns, filter rows, and summarise millions of log lines without loading them into anything.',
    tasksCount: 5,
  },
  {
    slug: 'log-hunting-with-grep',
    name: 'Log Hunting with grep',
    shortDescription:
      'A deploy went wrong and you are on-call. Hunt down errors in logs, search context around matches, and narrow a haystack to the one line that matters.',
    tasksCount: 5,
  },
  {
    slug: 'regular-expressions-bootcamp',
    name: 'Regular Expressions Bootcamp',
    shortDescription:
      'Patterns are everywhere — IPs in logs, emails in configs, timestamps in dumps. Learn to describe them precisely instead of guessing.',
    tasksCount: 6,
  },
  {
    slug: 'json-debrief-with-jq',
    name: 'JSON Debrief with jq',
    shortDescription:
      'The API dump is 10,000 lines of JSON. Extract fields, filter objects, and reshape the whole thing into something you can actually read.',
    tasksCount: 5,
  },
  {
    slug: 'deduplication-with-sort-uniq',
    name: 'Deduplication with sort & uniq',
    shortDescription:
      'Duplicate entries are clogging the pipeline. Deduplicate, rank by frequency, and find what appears where it should not.',
    tasksCount: 4,
  },
]
