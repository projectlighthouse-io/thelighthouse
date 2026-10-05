---
title: "Why Hashmap Isn't Always O(1)"
description: "Three measured benchmarks on an M3 Pro show a single hashmap insert 465,275x slower than the median, adversarial input scaling linearly, and arrays winning 60x."
published_at: 2026-04-14
draft: false
tags:
  - dsa
  - c
  - performance
  - hashmap
---

Every DSA class will tell you hashmap lookup is O(1). Every interview prep list repeats it. Every textbook prints it in a clean little complexity table. It is technically correct and operationally misleading.

In the [last article](/blog/why-an-o1-linked-list-is-80x-slower-than-an-on-array) we saw that Big-O hides the cost of memory access — an "O(1)" linked list insert was 44x slower than an "O(n)" array insert because the constant factor lived in the cache hierarchy. Hashmaps hide three more things in that same O(1) label, and each of them shows up as a measurable order-of-magnitude gap in the real world.

This article walks through a ~100-line C hashmap and four benchmarks, all run on an Apple M3 Pro. The first benchmark confirms the amortized story: per-operation time really does stay roughly flat as the table grows. The second measures the single worst insert in a million and finds it **465,275x slower than the median** — 19.5 milliseconds inside a loop that averages 42 nanoseconds. The third shows an attacker with 10,000 carefully chosen keys can force every lookup into a full linear scan of a bucket. The fourth benchmarks the happy path against a plain C array and finds the hashmap is **60x slower even when everything goes right**.

Those are the three asterisks the textbook forgets to print next to the "O(1)".

### What Big-O Is Actually Promising

Three words get conflated every time the complexity of a data structure comes up: **worst case**, **average case**, and **amortized**.

The worst case is the cost of the single most expensive call, over all possible inputs. For a separately-chained hashmap with a bad hash or adversarial keys, worst-case lookup is O(n) — you walk a linked list of every key that landed in the same bucket.

The average case is the cost of a single call averaged over some distribution of inputs. With a well-behaved hash function and random keys, hashmap lookup is O(1) on average, because the expected chain length stays bounded.

Amortized is something different. Amortized O(1) means the total cost of *any sequence* of n operations is O(n), so the per-operation cost averages to O(1) over the sequence. It does not mean every single operation is cheap. It means the occasional expensive operation is rare enough that it averages out. That is the promise. When a hashmap doubles its bucket array, that one insert pays for the rehash of everything before it, and the next million inserts pay nothing.

All three statements are true. None of them say what a p99 tail says. None of them say what an attacker says. None of them say what a tight array loop says. The claim "hashmap is O(1)" folds three separate guarantees into one sentence and drops the adversarial case entirely.

### A Hashmap in 100 Lines of C

Before benchmarking, we need something concrete to benchmark. Here is the whole data structure. It uses <term tip="Each bucket holds a linked list of entries whose keys hash to that bucket. The alternative is open addressing, where collisions are resolved by probing other buckets in the same flat array.">separate chaining</term>: each bucket is the head of a singly-linked list of entries whose keys collide in that slot.

```c
typedef struct Entry {
    char *key;
    int64_t value;
    struct Entry *next;
} Entry;

typedef struct HashMap {
    Entry **buckets;
    size_t nbuckets;
    size_t nentries;
    int resize_enabled;
} HashMap;
```

The hash function is <term tip="Fowler-Noll-Vo variant 1a. A fast non-cryptographic hash. Deterministic by design — no per-process seed — which is exactly the property the adversarial benchmark exploits.">FNV-1a</term>, which is about as simple as a usable hash function gets. The 64-bit FNV offset basis XORs in each byte and multiplies by the FNV prime:

```c
uint64_t fnv1a(const char *s) {
    uint64_t h = 0xcbf29ce484222325ULL;
    while (*s) {
        h ^= (unsigned char)*s++;
        h *= 0x100000001b3ULL;
    }
    return h;
}
```

Insertion hashes the key, masks the hash against `nbuckets - 1` to find the bucket (valid because we keep `nbuckets` a power of two), walks the chain looking for an existing entry with the same key, and prepends a new `Entry` node if none is found:

```c
int hm_put(HashMap *m, const char *key, int64_t value) {
    uint64_t h = fnv1a(key);
    size_t idx = h & (m->nbuckets - 1);

    for (Entry *e = m->buckets[idx]; e; e = e->next) {
        if (strcmp(e->key, key) == 0) {
            e->value = value;
            return 0;
        }
    }

    Entry *e = malloc(sizeof(Entry));
    if (!e) return -1;
    e->key = strdup(key);
    if (!e->key) { free(e); return -1; }
    e->value = value;
    e->next = m->buckets[idx];
    m->buckets[idx] = e;
    m->nentries++;

    if (m->resize_enabled &&
        (double)m->nentries / (double)m->nbuckets > LOAD_FACTOR_MAX) {
        if (hm_resize(m, m->nbuckets * 2) != 0) return -1;
    }

    return 0;
}
```

The last block is the amortized part. The <term tip="Ratio of entries to buckets. A low load factor keeps chains short at the cost of wasted memory. 0.75 is Java's default. Go uses 6.5 (denser). Rust's hashbrown uses ~0.875.">load factor</term> is `nentries / nbuckets`, and we keep it below `0.75`. Once we cross that threshold, we double the bucket array and re-hash every existing entry into the new buckets:

```c
static int hm_resize(HashMap *m, size_t new_nbuckets) {
    Entry **new_buckets = calloc(new_nbuckets, sizeof(Entry *));
    if (!new_buckets) return -1;

    for (size_t i = 0; i < m->nbuckets; i++) {
        Entry *e = m->buckets[i];
        while (e) {
            Entry *next = e->next;
            uint64_t h = fnv1a(e->key);
            size_t idx = h & (new_nbuckets - 1);
            e->next = new_buckets[idx];
            new_buckets[idx] = e;
            e = next;
        }
    }
    free(m->buckets);
    m->buckets = new_buckets;
    m->nbuckets = new_nbuckets;
    return 0;
}
```

That single loop — walk every bucket, rehash every entry, rewire every `next` pointer — is the thing amortized analysis has to smooth away. It runs in O(current table size). If the table has 786,432 entries, that loop does 786,432 hash computations, 786,432 modulo operations, and 786,432 pointer writes, all inside one `hm_put` call that the caller thought was O(1).

Why double, instead of growing by a fixed amount? Because doubling makes the work geometric. If you insert n items, the total rehash cost across all resizes is n + n/2 + n/4 + ... which sums to 2n. That 2n divided across n inserts is 2 units of work per insert — a constant. If you grew by a fixed 1000 instead, the rehash cost would be quadratic in n and the amortized bound would collapse. The doubling policy is not a style choice; it is the thing that makes the amortized claim true.

<blockquote class="font-sans! text-sm! not-italic! leading-relaxed!"><strong>Running the code:</strong> every benchmark in this article lives in <a href="https://github.com/projectlighthouse-io/run" target="_blank" rel="noopener noreferrer" class="text-black! font-semibold hover:underline">https://github.com/projectlighthouse-io/run&nbsp;<span class="text-xs opacity-60">↗</span></a> under <code>hashmap/</code>. Clone it and run <code>make run</code> from that directory if you'd rather not copy-paste each file. The inline code below is the exact source.</blockquote>

### Benchmark 1: Amortized O(1) Is Real — Sort Of

The first benchmark inserts N random-keyed entries for N in 100k, 500k, 1M, 5M, and measures total wall time. If the amortized claim holds, the per-op number should stay roughly flat as N grows by 50x. The harness is short:

```c
size_t sizes[] = {100000, 500000, 1000000, 5000000};
for (size_t si = 0; si < sizeof(sizes) / sizeof(sizes[0]); si++) {
    size_t N = sizes[si];
    HashMap *m = hm_new(16);

    char key[32];
    uint64_t t0 = now_ns();
    for (size_t i = 0; i < N; i++) {
        snprintf(key, sizeof(key), "k_%zu", i);
        hm_put(m, key, (int64_t)i);
    }
    uint64_t t1 = now_ns();

    double per_op = (double)(t1 - t0) / (double)N;
    printf("%10zu %14llu %12.2f\n", N, t1 - t0, per_op);
    hm_free(m);
}
```

Compile with `cc -O2 -std=c11 -Wall -Wextra -Wpedantic` and run. The literal output:

```
--- bench 1: amortized ---
         N       total_ns    ns_per_op
    100000       20010500       200.10
    500000       62734083       125.47
   1000000      118130750       118.13
   5000000      782004375       156.40
```

The per-op number is 200 ns at N=100k, drops to 118 ns by 1M, and bumps back up to 156 ns at 5M. This is the amortized claim behaving as advertised: the total cost grew by 39x when N grew by 50x. Per-operation time is nearly flat across a 50x range of input sizes. If the resize work weren't being smoothed away, the 1M and 5M numbers would dwarf the 100k number. They don't.

The 100k row is slightly slower per-op because it starts from 16 buckets and pays for proportionally more resizes early on — the first few doublings are a larger fraction of its total work. The 5M uptick is the more interesting one. By 5 million entries the hashmap's backing storage (8M buckets plus 5M entries with their string keys) has overflowed the M3 Pro's L3 cache, so every bucket load and every `strdup`'d key costs more in the cache hierarchy. The amortized cost is still bounded; the bound is just measured in slower nanoseconds.

This is the number the textbook is describing. It is genuinely useful. It is also the only one of the four benchmarks in this article that matches the O(1) claim.

### Benchmark 2: The Resize Spike Amortized Analysis Hides

Amortized O(1) says the average cost per insert is constant. It says nothing about the cost of any individual insert. To see what the average is hiding, instrument each insert with `clock_gettime` and record every latency into an array, then compute percentiles.

```c
for (size_t i = 0; i < N; i++) {
    snprintf(key, sizeof(key), "k_%zu", i);
    size_t buckets_before = hm_buckets(m);
    uint64_t t0 = now_ns();
    hm_put(m, key, (int64_t)i);
    uint64_t t1 = now_ns();
    uint64_t dt = t1 - t0;
    latencies[i] = dt;
    if (dt > max_lat) {
        max_lat = dt;
        max_idx = i;
        buckets_at_max = buckets_before;
        entries_at_max = hm_size(m) - 1;
    }
}

qsort(latencies, N, sizeof(uint64_t), cmp_u64);
uint64_t p50 = latencies[N / 2];
uint64_t p99 = latencies[(size_t)(N * 0.99)];
uint64_t p999 = latencies[(size_t)(N * 0.999)];
uint64_t p9999 = latencies[(size_t)(N * 0.9999)];
```

The output for N = 1,000,000 inserts:

```
--- bench 2: resize spike ---
N = 1000000 inserts, final buckets = 2097152
p50    (median)           = 42 ns
p99                        = 250 ns
p99.9                      = 875 ns
p99.99                     = 2334 ns
max                        = 19541583 ns  (insert #786432, growing from 1048576 -> 2097152 buckets with 786432 entries)
max / p50                  = 465275.8x
```

The median insert takes **42 nanoseconds**. The p99 takes **250 ns**, a clean 6x over the median — this is the "most inserts are cheap, occasionally a chain is a few entries long" distribution. The p99.9 is 875 ns, p99.99 is 2.3 microseconds. Then the tail falls off a cliff.

The single worst insert — number 786,432 out of a million — takes **19,541,583 nanoseconds**. That is **19.5 milliseconds**. That single `hm_put` call is **465,275x slower than the median**. Inside a loop where the caller has every reason to believe each iteration is O(1), one iteration ate 19.5 ms of wall clock.

The comment printed alongside the max tells you exactly what happened: the table was at 1,048,576 buckets holding 786,432 entries. That puts the load factor at `786432 / 1048576 = 0.75`, which is our threshold. The next insert pushed it over, and `hm_resize` allocated 2,097,152 new bucket pointers and rehashed every one of those 786,432 entries into the new array. Everything — the hash of every existing key, the masked index lookup, the pointer rewire — happens inside that one `hm_put` call.

The amortized claim is not wrong about this. Spread across the million inserts, 19.5 ms works out to 19.5 ns per insert, which is well within the 42 ns median budget. On average, the resize is free. On the particular iteration that triggered it, the resize is catastrophic.

Think about what this does to a real system. A request handler inserts into a cache. Most requests return in 42 ns from the cache layer. One request in a million returns after 19.5 ms — a 465,000x latency spike, right in the p99.9999 tail, perfectly correlated with load (the insert that pushes you over the threshold is the insert that happens when traffic is high). If you are writing a trading system, a game server, or anything with a latency SLA, this is the thing you were told did not exist.

Real-world hashmap implementations know this and handle it two ways. The first is incremental resize: start the new table, but rehash a few buckets per insert rather than all of them in one shot. Go's runtime does this — during a resize, each `hm_put` moves a small amount of the old table across, spreading the pain over many operations rather than stalling one. The second is to pre-size the table. If you know you'll insert a million entries, ask for a million-bucket table up front and skip every doubling.

The textbook "amortized O(1)" is not a lie. It is just averaging over a distribution whose tail is five orders of magnitude longer than its median, and refusing to tell you about the tail.

### Benchmark 3: Adversarial Input Forces O(n)

The amortized story assumes the keys you insert distribute roughly evenly across buckets. That assumption collapses when someone gets to choose the keys on purpose.

FNV-1a is deterministic. Given the function, given the table size, the bucket any string lands in is a pure function of the string. If an attacker knows the hash function (or can reverse-engineer it from observable behavior), they can brute-force strings that all hash to the same bucket modulo the table size, submit those strings as keys, and force every lookup into a linear walk of a giant linked list.

This benchmark finds 10,000 such strings and then measures lookup latency as the chain grows:

```c
const size_t nbuckets = 1024;
const size_t target_bucket = 0;

size_t max_keys = 10000;
char **keys = malloc(max_keys * sizeof(char *));
size_t found = 0;
size_t tried = 0;
while (found < max_keys) {
    char buf[32];
    snprintf(buf, sizeof(buf), "adv_%zu", tried++);
    if ((fnv1a(buf) & (nbuckets - 1)) == target_bucket) {
        keys[found++] = strdup(buf);
    }
}
```

Resize is pinned off (`m->resize_enabled = 0`) so the table stays at 1024 buckets no matter how many entries we shove in — otherwise growing the table would remap the colliding keys to new buckets and the attack would weaken. For each chain length, we insert that many adversarial keys and look up the one at the tail of the chain a thousand times:

```c
for (size_t ci = 0; ci < sizeof(chain_sizes) / sizeof(chain_sizes[0]); ci++) {
    size_t n = chain_sizes[ci];
    HashMap *m = hm_new(nbuckets);
    m->resize_enabled = 0;
    for (size_t i = 0; i < n; i++) hm_put(m, keys[i], (int64_t)i);

    const char *worst_key = keys[0];
    const size_t REPS = 1000;
    uint64_t t0 = now_ns();
    volatile int64_t sink = 0;
    for (size_t r = 0; r < REPS; r++) {
        int64_t out;
        hm_get(m, worst_key, &out);
        sink += out;
    }
    uint64_t t1 = now_ns();
    double avg = (double)(t1 - t0) / (double)REPS;
    printf("%10zu %18.2f %16llu\n", n, avg, t1 - t0);
    hm_free(m);
}
```

The output:

```
--- bench 3: adversarial (all keys collide) ---
generated 10000 colliding keys (tried 10288603 candidates, all hash to bucket 0 mod 1024)
 chain_len      lookup_ns_avg  lookup_ns_total
        10              25.08            25083
       100             224.38           224375
       500            1064.92          1064916
      1000            2092.54          2092542
      5000           10354.75         10354750
     10000           22138.38         22138375
```

The brute-force phase tried about 10.3 million candidates to find 10,000 that happened to hash to bucket 0 mod 1024 — roughly one in a thousand, which is exactly what you'd expect. On the M3 Pro that search took under a second. Finding collisions against a simple hash function is not a theoretical attack.

The lookup numbers scale linearly with chain length. 10 entries → 25 ns. 100 → 224 ns. 1,000 → 2,092 ns. 10,000 → 22,138 ns. Every 10x increase in chain length produces a 10x increase in lookup time. That is the textbook definition of O(n), and the cost per probe is remarkably consistent — roughly 2 nanoseconds per node, which is one cache miss plus a `strcmp` of short strings.

The hashmap has become a linked list with extra steps.

This is the shape of a hash-DoS attack, named and demonstrated by Crosby and Wallach in 2003. Submit a POST request with 10,000 form fields whose names all collide. The server parses the body into a hashmap keyed by field name. Every lookup during parsing is now O(n). A handler that used to run in microseconds now runs in tens of milliseconds. Do that from ten clients in parallel and you have taken the box down without generating any traffic an edge filter would flag.

The fix in production runtimes is a per-process random seed XORed into the hash function's initial state. Go, Rust, Python, and Java all do this. An attacker cannot precompute collisions because they don't know the seed, and they can't observe it easily from outside the process. This is the entire reason `HashMap` iteration order is randomized in those languages — the randomization is a security posture, and the iteration-order instability is a side effect that the language designers decided was worth protesting loudly about so you don't accidentally depend on it.

Our 100-line hashmap has no such seed. It is educational, not production, and the adversarial case is the easiest vulnerability in the world to reproduce. The point is that "average-case O(1)" is a statement about a distribution of inputs, and the moment an adversary chooses the distribution, the average case is no longer the case you're in.

### Benchmark 4: Even the Best Case Is 60x Slower Than an Array

Set aside resizes and adversaries. Assume the keys are well-distributed, the load factor is low, no chain is longer than a handful of entries. The hashmap is doing everything it's supposed to be doing. How does it compare to a plain C array for the same million int-to-int mapping?

```c
int64_t *arr = malloc(N * sizeof(int64_t));
for (size_t i = 0; i < N; i++) arr[i] = (int64_t)(i * 3 + 7);

HashMap *m = hm_new(1 << 21);
char key[32];
for (size_t i = 0; i < N; i++) {
    snprintf(key, sizeof(key), "k_%zu", i);
    hm_put(m, key, (int64_t)(i * 3 + 7));
}

volatile int64_t sink = 0;
uint64_t t0 = now_ns();
for (size_t r = 0; r < REPS; r++) {
    for (size_t i = 0; i < N; i++) sink += arr[i];
}
uint64_t t1 = now_ns();
uint64_t arr_total = t1 - t0;

t0 = now_ns();
for (size_t r = 0; r < REPS; r++) {
    for (size_t i = 0; i < N; i++) {
        snprintf(key, sizeof(key), "k_%zu", i);
        int64_t out;
        hm_get(m, key, &out);
        sink += out;
    }
}
t1 = now_ns();
uint64_t hm_total = t1 - t0;
```

The hashmap is over-sized to 2,097,152 buckets for a million entries, giving a load factor under 0.5 so chains stay short and the map operates squarely in its average case. The `sink` variable is `volatile` to keep `-O2` from deleting the whole loop. The output:

```
--- bench 4: array[] vs hashmap ---
op                                 total_ns    ns_per_op
array index                        12615250         1.26
hashmap get (string key)          756639375        75.66
ratio                        = 60.0x
```

One nanosecond, twenty-six picoseconds per array index. 75.66 nanoseconds per hashmap get. **Sixty times slower**, on the hashmap's best day.

Where does the 60x go? A hashmap lookup does, in order: build the key string with `snprintf`, hash the string with FNV-1a (one multiply and one XOR per byte), mask the 64-bit hash down to a bucket index, load the bucket pointer from the bucket array, chase a pointer to the first entry, load the entry's key pointer, `strcmp` it against the query key, and finally load the value. That's at least three pointer-chasing loads — bucket array, entry, key — each of which can miss in cache and stall for ~100 ns. The `strcmp` alone touches both key strings, which live in heap locations unrelated to the entry node.

The array index is one address arithmetic (`arr + i * 8`) plus one load. The prefetcher sees the sequential access pattern after two iterations and starts pulling cache lines in ahead of the loop. One 64-byte cache line holds eight `int64_t` values, so the actual RAM traffic is one line every eight iterations. The CPU's out-of-order engine runs far ahead of the memory unit. The summing loop becomes, effectively, a stream.

These two data structures are doing very different work. The hashmap can index by *anything* — strings, arbitrary structs, tuples. The array can only index by a small dense integer. When you have a small dense integer, the array is the right answer. The Linux kernel's PID table is an array. Scheduler run queues are arrays. Page frame arrays are arrays. File descriptor tables are arrays. Every hot-path lookup in the kernel where the key space is bounded and dense uses an array, because the 60x cost of a hashmap is intolerable in code that runs a million times a second.

This is not a knock on hashmaps. It is a reminder that generality has a price, and the price is paid on every lookup.

### What Real Runtimes Do

Each of the three failure modes above has a corresponding mitigation in a production hashmap implementation. Learning to spot them turns "the language's built-in map" from a black box into a comprehensible set of engineering choices.

**Go** addresses the resize spike and the adversarial case. Its runtime map uses a per-process random hash seed, xor'd into the hash function's initial state, so an attacker cannot precompute collisions. During a grow, the runtime doesn't rehash every bucket at once — each subsequent `mapassign` or `mapaccess` migrates a small number of buckets from the old table to the new one. The long tail of the insert distribution gets chopped up into many smaller taxes on nearby operations. Go's bucket layout also packs 8 key/value pairs into a cache-line-sized struct, which keeps the common "walk a short chain" case mostly in L1.

**Java's** `HashMap` addresses the adversarial case with a different trick. Once a bucket's chain grows past 8 entries, Java converts that single bucket from a linked list into a red-black tree. Lookup inside that bucket drops from O(chain length) to O(log chain length). An attacker submitting 10,000 colliding keys still degrades the map, but the degradation is O(log n) rather than O(n) — 14 comparisons instead of 10,000. The conversion threshold is tuned so the tree overhead only shows up when degradation is already happening.

**Rust's hashbrown**, the implementation behind `std::collections::HashMap`, is a port of Google's SwissTable. It attacks the cost-per-operation problem head-on. Instead of separate chaining, hashbrown uses open addressing with a control byte per slot and SIMD-probes 16 slots at a time with a single vector instruction. A lookup is a hash, a 128-bit vector compare against the control bytes, and — if there's a match — one key comparison. The constant factor is dramatically lower than the textbook separate-chaining approach, and the cache behavior is excellent because the control bytes and slots live in one contiguous region.

Each of those runtimes is patching a specific one of the three asterisks this article has been about. None of them can remove all three at once, because the three asterisks are consequences of the data structure itself, not implementation quirks.

### Closing

"O(1)" as written in DSA textbooks is shorthand for a much longer sentence. Spelled out, the sentence reads roughly: *amortized O(1) over a sequence of operations, assuming inputs drawn from a distribution the adversary does not control, with a constant factor on the order of 100 nanoseconds — versus the 1-nanosecond constant factor of an array.*

All of those caveats matter. The amortized guarantee is what lets you write `m[k] = v` without thinking; it also hides individual inserts that are 465,000x slower than the median. The average-case guarantee is what makes the data structure useful for unknown workloads; it also assumes nobody gets to choose the keys on purpose. The constant factor is the price of generality; it is also why the kernel's hot paths are arrays and not hash tables.

The rule of thumb that comes out of this: **if your key is a small dense integer, use an array.** If your workload has a predictable size, pre-size the map and skip every resize. If an untrusted party controls the keys, make sure the hash function is seeded per-process (Go, Rust, modern Java and Python all do this — roll your own at your peril). And when a p99 latency graph has a cliff at the exact point where your map would have doubled, you are looking at the tail that "amortized O(1)" taught you to ignore.

Big-O is a ceiling on growth rates. It is not a promise about nanoseconds, and it is not a promise about the worst case, and it is not a promise at all against someone who reads your source code.

#### Notes

If this is new territory, here are the canonical references to go deeper. Most of them reward a second pass once you have written your own hashmap.

<span class="text-black font-bold text-base">Core concepts (Wikipedia) </span>

<ol>
  <li><a href="https://en.wikipedia.org/wiki/Hash_table" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Hash table&nbsp;<span class="text-xs opacity-60">↗</span></a> — the data structure, separate chaining vs open addressing, load factor</li>
  <li><a href="https://en.wikipedia.org/wiki/Amortized_analysis" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Amortized analysis&nbsp;<span class="text-xs opacity-60">↗</span></a> — aggregate, accounting, and potential methods</li>
  <li><a href="https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">FNV hash function&nbsp;<span class="text-xs opacity-60">↗</span></a> — the hash used in this article</li>
  <li><a href="https://en.wikipedia.org/wiki/Collision_attack#Hash_flooding" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Hash flooding / hash-DoS&nbsp;<span class="text-xs opacity-60">↗</span></a> — the attack benchmark 3 demonstrates</li>
  <li><a href="https://en.wikipedia.org/wiki/Open_addressing" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Open addressing&nbsp;<span class="text-xs opacity-60">↗</span></a> — the alternative collision strategy used by SwissTable and hashbrown</li>
</ol>

<span class="text-black font-bold text-base">Deep dives </span>

<ol>
  <li>Crosby and Wallach, <em><a href="https://www.usenix.org/legacy/event/sec03/tech/full_papers/crosby/crosby.pdf" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Denial of Service via Algorithmic Complexity Attacks&nbsp;<span class="text-xs opacity-60">↗</span></a></em> (USENIX Security 2003) — the original hash-DoS paper. Short, clear, still relevant.</li>
  <li><em><a href="https://mitpress.mit.edu/9780262046305/introduction-to-algorithms/" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Introduction to Algorithms&nbsp;<span class="text-xs opacity-60">↗</span></a></em> (Cormen, Leiserson, Rivest, Stein), Chapter 11 — the rigorous treatment of hash tables, universal hashing, and the average-case analysis this article leans on.</li>
  <li><a href="https://go.dev/src/runtime/map.go" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">Go runtime/map.go&nbsp;<span class="text-xs opacity-60">↗</span></a> — the source of Go's built-in map, with extensive comments on incremental resize and bucket layout.</li>
  <li><a href="https://abseil.io/about/design/swisstables" target="_blank" rel="noopener noreferrer" class="text-black! font-medium hover:underline">SwissTable design notes&nbsp;<span class="text-xs opacity-60">↗</span></a> — Google's open-addressing + SIMD probing design, the basis for Rust's hashbrown.</li>
</ol>
