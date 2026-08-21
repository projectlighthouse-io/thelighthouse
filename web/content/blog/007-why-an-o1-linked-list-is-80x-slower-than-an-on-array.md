---
title: "Why an O(1) Linked List is 80x Slower Than an O(N) Array"
description: "A measured benchmark shows a sorted-array insert beating a linked-list insert by 44x, and a simple access-order change making the same linked list 80x slower. Big-O has nothing to do with either result."
published_at: 2026-04-11
draft: false
thumbnail_url: https://spaces.projectlighthouse.io/articles/oN_beats_o1.png
tags:
  - dsa
  - go
  - performance
  - cache
---

If you open a standard data structures textbook, performance is strictly a matter of counting operations. Under that model, a sorted array is an O(n) bottleneck because every insertion requires shifting elements. By contrast, a linked list offers a beautiful O(1) insertion because you only need to rewire a couple of pointers.

While these statements are correct, but the operation-count view does not, by itself, predict wall-clock time on modern hardware — and wall-clock time is a separate question that Big-O was not designed to answer.

This article walks through two benchmarks, run on an Apple M3 Pro. In the first, an O(n) sorted-array insert is **44x faster** than the textbook "O(1)" linked-list insert — a framing that only holds when you already have a pointer to the insertion point. Once you have to walk to that point, the linked-list insert is also O(n), and the constant factor is what matters. In the second, two identical linked-list traversals differ by **80x** — same nodes, same memory, same instructions — because the only thing that changed is the order in which the CPU visits them. The second result is the more important one. It is the shape of the problem that Big-O is not trying to describe.

```
ARRAY  (one contiguous run)

  0x100   0x108   0x110   0x118   0x120   0x128   0x130   0x138
  +-----+-+-----+-+-----+-+-----+-+-----+-+-----+-+-----+-+-----+
  |  7  | |  3  | |  9  | |  1  | |  5  | |  2  | |  8  | |  4  |
  +-----+-+-----+-+-----+-+-----+-+-----+-+-----+-+-----+-+-----+
  |<-------------- one 64-byte cache line -------------->|
  one load from RAM brings the next several elements along for free


LINKED LIST  ( elements spread across memory blocks )

    head
     |
     v
  0x2A8                   0x07C                    0xF40
  +-----+-----+           +-----+-----+            +-----+-----+
  |  7  |  *--+---------->|  3  |  *--+----+       |  9  |  *--+--> ...
  +-----+-----+           +-----+-----+    |       +-----+-----+
                                            |            ^
                                            +------------+

  every arrow is a data dependency the CPU cannot speculate around
  the prefetcher sees no stride and cannot help you
```

### What Big-O Actually Measures

**Big-O counts operations. It does not measure time**.

It assumes <span class="highlight-wavy">every operation takes the same unit of cost</span> — that reading an integer from memory is the same cost as comparing two integers, which is the same cost as writing a pointer. Under that assumption, an algorithm that performs n element moves is slower than one that performs 2 pointer writes as long as n is bigger than 2.

That assumption stopped holding decades ago. On a modern x86 or ARM core, the time it takes to read a value from memory <span class="highlight-wavy">depends entirely on where that value lives in the cache hierarchy</span>. A value in L1 cache costs roughly 1 nanosecond. A value in main memory costs roughly 100 nanoseconds. A cache-missing load can stall the CPU long enough to execute several hundred arithmetic instructions.

As mentioned above, Big-O measures operation count, which is a useful but a separate thing. While the operation count hasn't changed — what changed is the cost of each one. This is the abstraction working as designed; it is just now leaky enough that two algorithms with identical asymptotic complexity can differ by 100x in practice. Big-O is necessary but not painting the full picture when hardware is involved.

### The Memory Hierarchy in Five Numbers

These numbers are worth having in mind for backend work. They are approximate — they vary by chip, vendor, and generation — but the order of magnitude has been stable for twenty years.

| Level | Latency | Cycles | What lives here |
|-------|---------|--------|-----------------|
| L1 cache | ~1 ns | ~4 | the few KB you're actively working on |
| L2 cache | ~4 ns | ~12 | recently used data, per-core |
| L3 cache | ~12 ns | ~40 | shared across cores, tens of MB |
| Main memory (RAM) | ~100 ns | ~300 | everything else |
| NVMe SSD | ~100 μs | ~300,000 | swap, mmapped files |

The gap between L1 and RAM is roughly 100x. This is why modern CPUs spend enormous amounts of silicon on caches, branch predictors, and prefetchers: the execution units are so fast that they spend most of their life waiting for data.

One number is not in that table but matters a great deal. Memory moves from RAM into cache in fixed-size chunks called <term tip="The atomic unit of transfer between RAM and cache on modern CPUs. On x86 and ARM64, almost always 64 bytes. When you read a single byte, you pay for the whole line.">cache lines</term>, and on the x86 and ARM64 chips you are likely to run on, one line is 64 bytes.

When you read <span class="highlight-wavy">a single `int`, the CPU does not read 8 bytes. It reads 64, and the other 56 come along at zero extra cost </span>. That is one of the main reasons arrays are fast in practice.

### What Arrays Get Right

Consider a Go `[]int` containing a million elements. Those elements live in one contiguous 8 MB block of memory. The addresses of consecutive elements differ by exactly 8 bytes.

When you walk the slice from front to back, the CPU sees you touch address `X`, then `X+8`, then `X+16`. That stride is the simplest thing in the world for the <term tip="A hardware unit that watches your memory access pattern and speculatively loads data it thinks you'll ask for next. It's extraordinarily good at detecting simple strides.">hardware prefetcher</term> to detect.

After a few iterations, the prefetcher notices you are marching forward and starts pulling the next cache line into L1 before you even ask for it. Three things happen as a result. Your reads hit L1 almost every time. The fetches that do go to RAM happen in parallel with your computation, not blocking it. And you use every byte of every cache line you paid for, because the data is packed tight — one cache line holds eight 64-bit integers, so one 100 ns round trip to RAM buys you eight free reads.

This is why linear scans of large arrays are fast in practice. Summing a 10-million-element slice of 64-bit integers is 80 MB of memory traffic, but the prefetcher turns it into a steady stream and the CPU's ALUs stay mostly busy adding while the bus feeds them data.

### What Linked Lists (Can) Get Wrong ... Sometimes

Now consider a singly-linked list with the same million integers. Each node is a separate heap allocation made at some arbitrary time in the program's life. The allocator placed them wherever it had space.

When you walk the list, the CPU reads an address, then reads a completely different address, then reads yet another. **The prefetcher sees no stride, but that's not the most important thing** (in context of this article). The important thing here is: the address of node B is stored inside node A. The CPU has to finish reading A before it can even know what to fetch next.

This is a data dependency the CPU cannot speculate around. Modern out-of-order execution engines hide memory latency by running ahead — fetching the next instruction's operands while the current one is still waiting. That trick works when the CPU can predict "next." With a linked list, each `.next` load is a mystery until the previous load completes. Every hop is a serialized, blocking fetch that pays whatever latency that node costs — L1 if you get lucky, L3 or RAM when you don't — and the CPU cannot start it early.

There is a subtler cost. Each node is small — payload plus pointer, maybe 16 or 24 bytes. When the CPU fetches that node, it still pays for the full 64-byte cache line. The other 40 bytes are whatever random heap garbage happens to be adjacent: a different allocation, a freed slot, padding. You paid for 64 bytes of bandwidth and used 24. In the array case, all 64 were useful data.

So the linked list can lose on three axes at once: every hop is a serial dependency the CPU cannot speculate around, the prefetcher cannot help, and even when a fetch succeeds it wastes most of the bandwidth it paid for. Standard complexity analysis does not directly model any of this.

The word "can" matters. Whether the linked list loses depends on whether its nodes are actually scattered in memory. The next two benchmarks show what happens at both extremes.

<blockquote class="font-sans! text-sm! not-italic! leading-relaxed!"><strong>Running the code:</strong> every benchmark in this article lives in <a href="https://github.com/projectlighthouse-io/run" target="_blank" rel="noopener noreferrer" class="text-black! font-semibold hover:underline">https://github.com/projectlighthouse-io/run&nbsp;<span class="text-xs opacity-60">↗</span></a> under <code>arrays-vs-linked-lists/</code>. Clone it and run <code>go test -bench=. -benchmem -benchtime=10x</code> from that directory if you'd rather not copy-paste each file. The inline code below is the exact source.</blockquote>

### Benchmark 1: Sorted Insertion

The first benchmark inserts 100,000 random integers into a sorted collection — once as a `[]int` slice, once as a hand-rolled singly-linked list. Paste this into a file called `sorted_insert_test.go` and run it with `go test -bench=. -benchmem -benchtime=10x`.

```go
package arrayslinked

import (
	"math/rand"
	"slices"
	"testing"
)

const N = 100_000

type Node struct {
	Val  int
	Next *Node
}

// insertSlice inserts v into a sorted []int and returns the new slice.
// slices.Insert (Go 1.21+) is the idiomatic modern form and handles the
// overlapping move internally.
func insertSlice(s []int, v int) []int {
	i, _ := slices.BinarySearch(s, v)
	return slices.Insert(s, i, v)
}

// insertLinked inserts v into a sorted singly-linked list.
// returns the (possibly new) head.
func insertLinked(head *Node, v int) *Node {
	n := &Node{Val: v}
	if head == nil || head.Val >= v {
		n.Next = head
		return n
	}
	cur := head
	for cur.Next != nil && cur.Next.Val < v {
		cur = cur.Next
	}
	n.Next = cur.Next
	cur.Next = n
	return head
}

// deterministic input so both benchmarks see the same numbers.
func makeInput() []int {
	r := rand.New(rand.NewSource(42))
	xs := make([]int, N)
	for i := range xs {
		xs[i] = r.Int()
	}
	return xs
}

func BenchmarkSortedInsertSlice(b *testing.B) {
	input := makeInput()
	b.ResetTimer()
	for b.Loop() {
		s := make([]int, 0, N)
		for _, v := range input {
			s = insertSlice(s, v)
		}
	}
}

func BenchmarkSortedInsertLinked(b *testing.B) {
	input := makeInput()
	b.ResetTimer()
	for b.Loop() {
		var head *Node
		for _, v := range input {
			head = insertLinked(head, v)
		}
	}
}
```

The slice is pre-sized with `make([]int, 0, N)` so the benchmark isn't penalized for `append` reallocations — we're measuring insertion, not growth. The linked list gets no such help because a separate allocation per node is inherent to the data structure. `for b.Loop()` is the Go 1.22+ benchmark form; it replaces the old `for i := 0; i < b.N; i++` pattern and avoids a class of subtle bugs around `b.N`. `slices.Insert` is the Go 1.21+ stdlib helper for inserting into a sorted slice, which handles the overlapping `copy` internally.

Run it:

```bash
go test -bench=SortedInsert -benchmem -benchtime=10x
```

Actual output on an Apple M3 Pro:

```
goos: darwin
goarch: arm64
pkg: arrayslinked
cpu: Apple M3 Pro
BenchmarkSortedInsertSlice-11        10     330_817_438 ns/op     802_816 B/op         1 allocs/op
BenchmarkSortedInsertLinked-11       10  14_773_428_496 ns/op   1_600_025 B/op   100_000 allocs/op
PASS
```

Three columns matter. `ns/op` is total nanoseconds per benchmark iteration, and each iteration inserts 100,000 elements — so the slice averages about 3,308 ns per insertion and the linked list about 147,734 ns. The slice is **44.6x faster** in wall-clock time.

`B/op` is bytes allocated per iteration: the slice allocates 800 KB in a single contiguous run (the pre-sized backing array never resizes), the linked list allocates 1.6 MB across 100,000 separate nodes. `allocs/op` is the number of distinct heap allocations: the slice makes exactly one; the linked list makes one per element.

Every column tells the same story. The slice is winning on wall time, on total memory, and on allocator pressure.

The slice's find step is a binary search over contiguous memory — a tight loop that stays in L1. The slice's shift step is an overlapping move that Go lowers to a vectorized `memmove`, which the CPU loves and executes at memory bandwidth. The linked list's find step is a pointer chase: on each insert, it walks an average of N/2 nodes, and each hop is potentially a full-latency memory stall. The pointer rewiring at the end is free; it is the walk that dominates.

Big-O told us the slice insert is O(n) for the shift and the linked list insert is O(n) for the walk plus O(1) for the splice. In asymptotic terms these are the same. In measured terms one is 44x the other. The difference lives in the constant factor that Big-O deliberately throws away.

### Benchmark 2: Pure Traversal, Two Access Orders

Insertion mixes several costs. To see the cache effect in isolation, sum the elements — no inserts, no allocations during the benchmark, just a linear scan.

This benchmark has three variants: a slice, a linked list where the nodes are traversed in allocation order, and a linked list where the traversal order has been shuffled so it no longer matches the allocation order. The third one is the interesting case.

```go
package arrayslinked

import (
	"math/rand"
	"testing"
)

const M = 10_000_000

type TNode struct {
	Val  int
	Next *TNode
}

// Sink is a package-level variable written to inside the benchmark loops
// to keep the compiler from eliminating the work. Without it, the Go
// compiler can notice that `sum` has no observable effect and delete the
// loop entirely — the single most common Go benchmarking mistake.
var Sink int

func buildTSlice() []int {
	r := rand.New(rand.NewSource(1))
	xs := make([]int, M)
	for i := range xs {
		xs[i] = r.Int()
	}
	return xs
}

// buildTList allocates all nodes in a tight loop. In a fresh heap, Go's
// allocator tends to place these nodes adjacent in memory (not guaranteed
// — depends on size class, GC phase, and heap fragmentation), so the
// traversal order matches the allocation order and the prefetcher handles
// it almost as well as the slice. This is the linked list's best case —
// the one most microbenchmarks accidentally measure.
func buildTList() *TNode {
	r := rand.New(rand.NewSource(1))
	var head *TNode
	for i := 0; i < M; i++ {
		head = &TNode{Val: r.Int(), Next: head}
	}
	return head
}

// buildTListShuffled allocates M nodes, shuffles them, and wires the .next
// pointers in the shuffled order. The nodes still live in whatever memory
// the allocator gave them — but now the traversal order jumps between them
// randomly, defeating the prefetcher. This models the realistic case where
// a long-running program's heap has been churned and traversal order no
// longer matches allocation order.
func buildTListShuffled() *TNode {
	r := rand.New(rand.NewSource(1))
	nodes := make([]*TNode, M)
	for i := 0; i < M; i++ {
		nodes[i] = &TNode{Val: r.Int()}
	}
	shuf := rand.New(rand.NewSource(2))
	shuf.Shuffle(M, func(i, j int) {
		nodes[i], nodes[j] = nodes[j], nodes[i]
	})
	for i := 0; i < M-1; i++ {
		nodes[i].Next = nodes[i+1]
	}
	return nodes[0]
}

func BenchmarkSumSlice(b *testing.B) {
	xs := buildTSlice()
	b.ResetTimer()
	for b.Loop() {
		sum := 0
		for _, v := range xs {
			sum += v
		}
		Sink = sum
	}
}

func BenchmarkSumLinkedAdjacent(b *testing.B) {
	head := buildTList()
	b.ResetTimer()
	for b.Loop() {
		sum := 0
		for n := head; n != nil; n = n.Next {
			sum += n.Val
		}
		Sink = sum
	}
}

func BenchmarkSumLinkedShuffled(b *testing.B) {
	head := buildTListShuffled()
	b.ResetTimer()
	for b.Loop() {
		sum := 0
		for n := head; n != nil; n = n.Next {
			sum += n.Val
		}
		Sink = sum
	}
}
```

Actual output on the same M3 Pro:

```
goos: darwin
goarch: arm64
pkg: arrayslinked
cpu: Apple M3 Pro
BenchmarkSumSlice-11              5      14_938_100 ns/op
BenchmarkSumLinkedAdjacent-11     5      12_801_017 ns/op
BenchmarkSumLinkedShuffled-11     5   1_022_750_967 ns/op
PASS
```

If we read those numbers carefully, the slice sums 10 million integers in **14.9 ms**. The linked list, traversed in allocation order, sums the same 10 million integers in **12.8 ms** — indistinguishable from the slice, and actually faster by a hair in this run.

The linked list, traversed in shuffled order, sums the same 10 million integers in **1,022 ms** — **80x slower** than the same nodes traversed in allocation order, and **68x slower** than the slice.

Nothing changed between the adjacent and shuffled versions except the order in which the CPU visits the nodes. The nodes are identical. Their addresses in memory are identical. The sum is identical. The code in the inner loop is identical — `for n := head; n != nil; n = n.Next`.

The only difference is that in the adjacent case, `head.Next` points to the node one memory word ahead, and the prefetcher learns the stride after a few iterations. In the shuffled case, `head.Next` points to a random node somewhere else in the million-node heap, and the prefetcher can't predict anything, so every hop is a serial cache miss.

This is the result that matters. Not "arrays are faster than linked lists" — that framing obscures the point. The point is: **the CPU's effective speed is set by access order, and contiguous arrays guarantee the good access order for free.**

A linked list can achieve the same speed if its traversal order happens to match its allocation order. In a fresh-heap microbenchmark, this is free. In a long-running program whose heap has been churned, it is almost never true.

The adjacent-order linked list is the best case for a linked list. The shuffled one is what a linked list becomes after a few hours of production traffic. The 80x gap between them is the answer to the question "why does this microbenchmark say my linked list is fine, but production tells me otherwise?"

Standard complexity analysis models none of this. Both linked list benchmarks perform the same O(M) traversal. The constant factor is 80x different.

### When Linked Lists Actually Win

The honest answer is: rarely, and almost never for the reason usually given. The "cheap middle insertion" framing holds when you already have a pointer to the insertion site. It does not hold when you have to walk to that site, because the walk dominates the cost.

There are real use cases for linked lists, but they tend to look different from the usual introductory framing.

The strongest case is the <term tip="A linked list where the 'next' pointers are embedded inside the objects being linked, rather than in separately-allocated wrapper nodes. The kernel's `list_head` is the canonical example.">intrusive linked list</term> pattern the Linux kernel uses extensively via `struct list_head`. Intrusive lists embed the `next` and `prev` pointers directly inside the objects being linked, so there is no separate wrapper allocation.

The objects are already allocated for their own reasons; the list is just a threading of them, and you can splice an object in and out of multiple lists simultaneously. This is how the kernel tracks pages on both a "dirty" list and an "active" list without duplicating them.

LRU caches are a second case. The linked list is a reordering structure over objects already held in a hash map. You never walk the list — you splice at positions you got from the map. The O(1) splice is the whole point, and you never eat the walk cost.

Schedulers and run queues are a third. A scheduler holding a pointer to the currently-running node can remove it and continue iterating without invalidating the walk — something an array would make awkward, because removing the current element requires shifting everything after it.

Which changes the index you were iterating on. Intrusive doubly-linked lists sidestep this entirely: unlink the current node, move to the next pointer you already saved, keep going.

The unifying principle: linked lists are useful when you already have a pointer to the node you want to rewire, and when the allocation of the underlying objects is controlled for reasons unrelated to the list.

They are not a great fit as a general-purpose sequence container, and the often-cited intuition that they are cheaper than arrays for insert-heavy workloads does not hold up when you measure wall-clock time on modern hardware.

### What This Means For Your Code

Three practical rules come out of this, useful to keep alongside Big-O for day-to-day work.

<span class="text-black font-bold">First</span>, default to contiguous data structures. Prefer `[]T` to `[]*T` when `T` is a value type you own, and prefer `[]*T` to any linked structure for almost all sequence use cases. The difference between `[]Point` and `[]*Point` tends to be larger than it first appears. `[]Point` stores the values inline in one contiguous run. `[]*Point` stores the pointers contiguously but the actual `Point` values are scattered across the heap, one allocation each, and iterating them is a pointer chase.

The slice of pointers still beats a linked list — the pointers themselves are packed and the iteration has no data dependencies between fetches because the CPU can speculate forward through the pointer array — but if you can live with value semantics, `[]T` is almost always the right answer.

<span class="text-black font-bold">Second</span>, think about <span class="highlight-wavy">struct-of-arrays versus array-of-structs</span> when you find yourself repeatedly looping over one field of a struct. Consider:

```go
type Particle struct {
	X, Y, Z    float64
	VX, VY, VZ float64
	Mass       float64
	Charge     float64
	ID         uint64
}

particles := make([]Particle, 1_000_000)

// loop that only reads X
for i := range particles {
	total += particles[i].X
}
```

Each `Particle` is 72 bytes. When you read `particles[i].X`, you pull in a 64-byte cache line holding the X you wanted plus Y, Z, VX, some of VY — none of which this loop uses. You paid for 64 bytes of bandwidth and used 8. Useful bandwidth: 12%.

The struct-of-arrays alternative splits the fields into parallel slices:

```go
type Particles struct {
	X, Y, Z    []float64
	VX, VY, VZ []float64
	Mass       []float64
	Charge     []float64
	ID         []uint64
}
```

Now the `X` slice is one contiguous run of nothing but X values. A 64-byte cache line holds eight of them. Useful bandwidth is 100%, and the loop runs several times faster. The trade-off is a slightly awkward access pattern when you need multiple fields of the same particle, but in numerical and analytics workloads SoA dominates for this reason.

<span class="text-black font-bold">Third</span>, keep Big-O. It is the ceiling on growth rates and it is as useful as it ever was — it will still reliably rule out the O(n²) algorithm that turns a million-element input into an hour of runtime. Layer cache-aware intuition on top of it. Between two algorithms with the same asymptotic complexity, Big-O is not trying to tell you which will be faster — that is a different question, and the constant factor can easily be 100x. That constant factor is often the cache hierarchy.

### A Note On Pointer-Heavy Languages

Everything above has been in Go, where `[]int` really is what it looks like: a contiguous run of 64-bit integers. That is not true in every language, and it is easy to trip on this when crossing ecosystems.

Java's `ArrayList<Integer>` is an array of references to heap-allocated `Integer` boxes. Python's list holds `PyObject` pointers to boxed values. C#'s `List<object>` behaves the same for boxed value types. In all three cases, "array" has the cache profile of a linked list, not a true contiguous buffer.

This is why those languages ship specialized primitive containers. Java's `int[]` is a true contiguous array of 32-bit integers. Python engineers reach for `array.array` or, much more commonly, NumPy, whose `ndarray` is a contiguous C buffer wearing a Python wrapper. C# has `Span<int>` and native arrays.

In a language with boxed primitives, the primitive container is not optional — it is the only shape of "array" that the hardware will help you with.

### Closing

Big-O is correct. But it isn't the whole picture. The memory hierarchy is another layer of the cost model — and on modern hardware, it is often the layer that decides how fast the code runs in practice.

In standard day-to-day applications, you might never notice this difference. But if you are optimizing code at scale, writing a compiler, or working in environments where a simple Hello World JSON response triggers dozens of function calls and heap allocations, this intuition becomes mandatory.

The sharpest version of the lesson comes from the shuffled-vs-adjacent linked-list benchmark: same nodes, same memory, same instructions, 80x different runtime. That gap has nothing to do with operation count and everything to do with whether the CPU can predict what you are about to ask for next.

If you want to see it directly rather than infer it from wall time, the next step is to run the same benchmarks under perf stat -e cache-misses,cache-references on Linux, or under Instruments' Cache profiler on macOS. The cache misses line up with the wall-clock gap almost exactly. That is the subject of another article.

Want the systematic version of this, including where arrays show up under the hood of every other data structure you'll meet? The [DSA Fundamentals book](/books/dsa-fundamentals) walks through it end to end — from RAM up through trees, heaps, and graph algorithms, with this same hardware-aware lens.

#### Notes

If this was new to you, here are the canonical resources to go deeper. Bookmark them — most of this material repays multiple passes as your mental model matures.

<span class="text-black font-bold text-base">Core concepts (Wikipedia) </span>

<ol>
  <li><a href="https://en.wikipedia.org/wiki/CPU_cache" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">CPU cache&nbsp;<span class="text-xs opacity-60">↗</span></a> — how the L1/L2/L3 hierarchy is organized</li>
  <li><a href="https://en.wikipedia.org/wiki/Cache_prefetching" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Cache prefetching&nbsp;<span class="text-xs opacity-60">↗</span></a> — hardware and software prefetchers, stride detection</li>
  <li><a href="https://en.wikipedia.org/wiki/Memory_hierarchy" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Memory hierarchy&nbsp;<span class="text-xs opacity-60">↗</span></a> — the cost model at a glance</li>
  <li><a href="https://en.wikipedia.org/wiki/Out-of-order_execution" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Out-of-order execution&nbsp;<span class="text-xs opacity-60">↗</span></a> — why the CPU wants to run ahead, and what blocks it</li>
  <li><a href="https://en.wikipedia.org/wiki/AoS_and_SoA" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">AoS and SoA&nbsp;<span class="text-xs opacity-60">↗</span></a> — struct-of-arrays vs array-of-structs</li>
  <li><a href="https://en.wikipedia.org/wiki/Linked_list" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Linked list&nbsp;<span class="text-xs opacity-60">↗</span></a> — including intrusive variants</li>
</ol>

<span class="text-black font-bold text-base">Deep dives </span>

<ol>
  <li><em><a href="https://www.akkadia.org/drepper/cpumemory.pdf" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">What Every Programmer Should Know About Memory&nbsp;<span class="text-xs opacity-60">↗</span></a></em> by Ulrich Drepper — the canonical long-form treatment. 114 pages; worth it.</li>
  <li><a href="https://www.agner.org/optimize/" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Agner Fog's optimization manuals&nbsp;<span class="text-xs opacity-60">↗</span></a> — the definitive reference for x86 microarchitecture, instruction latencies, and pipeline behavior.</li>
  <li>Mike Acton, <em>Data-Oriented Design and C++</em> (CppCon 2014) — the argument that memory layout is the job. Search YouTube.</li>
</ol>
