## Hello World

Every C program starts at `main`. The return value goes back to the OS — 0 means success, anything else signals an error.

```c
#include <stdio.h>  // pulls in printf, puts, etc. — angle brackets search system paths
#include "myheader.h"  // quotes search the current directory first, then system paths

int main(int argc, char *argv[]) {
    // argc = argument count (always >= 1, the program name)
    // argv = array of null-terminated strings
    // argv[argc] is guaranteed to be NULL

    printf("hello, world\n");  // \n flushes stdout on most systems — without it, output may buffer
    puts("hello again");       // puts() appends a newline automatically

    return 0;  // EXIT_SUCCESS from <stdlib.h> is the portable way
}
```

## Variables and Types

C types have platform-dependent sizes. The only guarantee: `sizeof(char) == 1`. Use `<stdint.h>` when you need exact widths.

```c
#include <stdio.h>
#include <stdint.h>   // fixed-width types — use these for serialization, protocols, hardware
#include <stdbool.h>  // bool, true, false (C99+)
#include <limits.h>   // INT_MAX, UINT_MAX, etc.
#include <float.h>    // DBL_MAX, FLT_EPSILON, etc.

int main(void) {
    // basic types — sizes vary by platform
    char c = 'A';            // at least 8 bits. signedness is implementation-defined!
    signed char sc = -1;     // explicitly signed: -128 to 127
    unsigned char uc = 255;  // explicitly unsigned: 0 to 255

    short s = 32767;         // at least 16 bits
    int i = 42;              // at least 16 bits, usually 32
    long l = 100000L;        // at least 32 bits
    long long ll = 1LL << 40;  // at least 64 bits (C99+)

    float f = 3.14f;         // ~7 decimal digits precision
    double d = 3.14159265;   // ~15 decimal digits — prefer this over float
    // long double exists but its size varies wildly (80-bit, 128-bit, or same as double)

    // sizeof returns bytes, type is size_t (unsigned)
    printf("int: %zu bytes\n", sizeof(int));
    printf("pointer: %zu bytes\n", sizeof(void *));  // 4 on 32-bit, 8 on 64-bit

    // fixed-width types — portable and explicit
    int8_t   exact_8  = -128;
    uint16_t exact_16 = 65535;
    int32_t  exact_32 = -2147483648;
    uint64_t exact_64 = UINT64_MAX;

    // size_t for sizes/indices, ptrdiff_t for pointer differences
    size_t len = 42;         // unsigned, big enough for any object size
    ptrdiff_t diff = &i - &i;  // signed result of pointer subtraction

    // _Bool or bool (with stdbool.h) — 0 is false, anything else is true
    bool flag = true;

    // const — the compiler enforces this, but it's not "deep" for pointers
    const int MAX = 100;
    // MAX = 200;  // compile error

    return 0;
}
```

## Operators

C operator precedence has a few traps inherited from B. When in doubt, use parentheses.

```c
#include <stdio.h>

int main(void) {
    // arithmetic — nothing surprising here
    int a = 10 / 3;    // 3 — integer division truncates toward zero
    int b = -10 / 3;   // -3 in C99+ (was implementation-defined in C89)
    int c = 10 % 3;    // 1 — modulo follows division truncation

    // bitwise — essential for flags, protocols, embedded
    unsigned flags = 0;
    flags |= (1 << 3);         // set bit 3
    flags &= ~(1 << 3);        // clear bit 3
    flags ^= (1 << 3);         // toggle bit 3
    int is_set = (flags >> 3) & 1;  // test bit 3

    // shifting: left shift by N = multiply by 2^N
    // right shift of signed values is implementation-defined — avoid it
    unsigned x = 1u << 31;  // always use unsigned for bit manipulation

    // logical — short-circuit evaluation, same as most languages
    int p = 1, q = 0;
    if (p && q) { /* q is evaluated */ }
    if (p || q) { /* q is NOT evaluated because p is truthy */ }

    // ternary
    int max = (a > b) ? a : b;

    // comma operator — evaluates left to right, result is the rightmost
    int val = (1, 2, 3);  // val == 3 — rarely useful outside for-loops

    // precedence traps:
    // & has LOWER precedence than ==
    if ((flags & 0x04) == 0x04) { }  // correct — parens required
    // if (flags & 0x04 == 0x04) { }  // BUG: parses as flags & (0x04 == 0x04)

    // -> and . bind tighter than * (dereference)
    // *ptr.field means *(ptr.field), not (*ptr).field — use ptr->field

    // sizeof is NOT a function for types, but parens are conventional
    int sz = sizeof(int);     // parens required for types
    int sy = sizeof a;        // parens optional for variables (but just use them anyway)

    return 0;
}
```

## Strings

C strings are just `char` arrays terminated by `'\0'`. Every string function depends on that null byte — forget it and you get buffer overruns.

```c
#include <stdio.h>
#include <string.h>  // strlen, strcpy, strcmp, strcat, memcpy, memset

int main(void) {
    // string literals are stored in read-only memory
    const char *greeting = "hello";  // pointer to read-only data — do NOT modify
    char buf[32] = "hello";          // copies literal into a mutable stack buffer

    // the compiler adds '\0' automatically for literals
    // "hello" is actually 6 bytes: h, e, l, l, o, \0
    printf("strlen: %zu\n", strlen(buf));   // 5 — does NOT count the null terminator
    printf("sizeof: %zu\n", sizeof(buf));   // 32 — the full buffer size

    // copying — always prefer strncpy or snprintf to avoid overflows
    char dest[16];
    strncpy(dest, buf, sizeof(dest) - 1);  // leave room for null terminator
    dest[sizeof(dest) - 1] = '\0';         // strncpy does NOT guarantee null termination!

    // snprintf is the safest option — always null-terminates, returns what it WOULD have written
    char msg[64];
    int needed = snprintf(msg, sizeof(msg), "user: %s, id: %d", "alice", 42);
    if ((size_t)needed >= sizeof(msg)) {
        // output was truncated
    }

    // comparison — strcmp returns 0 for equal (NOT a boolean true)
    if (strcmp(buf, "hello") == 0) {
        printf("equal\n");
    }
    // strncmp compares at most N characters — useful for prefix matching
    if (strncmp(buf, "hel", 3) == 0) {
        printf("starts with hel\n");
    }

    // concatenation
    char path[256] = "/home/";
    strncat(path, "user", sizeof(path) - strlen(path) - 1);

    // searching
    char *found = strstr("hello world", "world");  // pointer to "world" or NULL
    char *chr = strchr("hello", 'l');               // pointer to first 'l'

    // raw memory operations — work on any data, not just strings
    char block[64];
    memset(block, 0, sizeof(block));          // zero out memory
    memcpy(block, buf, strlen(buf) + 1);      // copy including null terminator
    // memmove handles overlapping regions — memcpy does NOT

    return 0;
}
```

## Arrays

Arrays in C decay to pointers when passed to functions — you lose size information. Always pass the length separately.

```c
#include <stdio.h>

int main(void) {
    // stack-allocated, zero-initialized
    int nums[5] = {0};  // all elements set to 0

    // partial initialization — remaining elements are zero
    int vals[5] = {1, 2, 3};  // vals[3] == 0, vals[4] == 0

    // compiler-determined size
    int auto_sized[] = {10, 20, 30};  // size is 3
    size_t count = sizeof(auto_sized) / sizeof(auto_sized[0]);  // common pattern to get array length

    // designated initializers (C99+) — set specific indices
    int sparse[100] = {
        [0] = 1,
        [50] = 2,
        [99] = 3,
    };

    // multi-dimensional — stored in row-major order (row elements are contiguous)
    int matrix[3][4] = {
        {1, 2, 3, 4},
        {5, 6, 7, 8},
        {9, 10, 11, 12},
    };
    // matrix[1][2] is at memory offset (1 * 4 + 2) * sizeof(int)

    // arrays decay to pointers in most contexts
    int *p = nums;             // equivalent to &nums[0]
    printf("%d\n", *(p + 2));  // same as nums[2] — pointer arithmetic scales by element size
    printf("%d\n", p[2]);      // [] is just syntactic sugar for *(p + offset)

    // VLA — variable length arrays (C99, optional in C11+)
    // stack-allocated, no way to check for allocation failure — avoid for large sizes
    int n = 10;
    int dynamic[n];  // allocated on the stack at runtime
    for (int i = 0; i < n; i++) {
        dynamic[i] = i * i;
    }

    // C has NO bounds checking — this compiles and silently corrupts memory:
    // nums[10] = 42;  // undefined behavior, writing past the end

    return 0;
}
```

## Pointers

Pointers are C's most powerful and most dangerous feature. They hold memory addresses — nothing more, nothing less.

```c
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int x = 42;
    int *p = &x;        // & = "address of" — p now holds x's memory address
    printf("%d\n", *p); // * = "dereference" — follow the pointer to read the value: 42

    *p = 100;            // write through the pointer — x is now 100

    // pointer arithmetic — scales by the pointed-to type's size
    int arr[5] = {10, 20, 30, 40, 50};
    int *q = arr;        // arrays decay to pointers
    q += 2;              // advances by 2 * sizeof(int) bytes
    printf("%d\n", *q);  // 30

    // pointer difference gives element count, not byte count
    ptrdiff_t diff = q - arr;  // 2

    // NULL — the "points to nothing" sentinel
    int *null_ptr = NULL;
    // dereferencing NULL is undefined behavior (usually a segfault)
    // always check: if (ptr != NULL) { ... }

    // void* — generic pointer, can hold any data pointer
    // you MUST cast before dereferencing (the compiler doesn't know the type)
    void *generic = &x;
    int *typed = (int *)generic;  // C allows implicit conversion, but be explicit

    // const correctness with pointers — read right to left
    const int *cp = &x;     // pointer to const int — can't modify *cp
    int *const pc = &x;     // const pointer to int — can't modify pc itself
    const int *const cpc = &x;  // both const

    // double pointer — common for functions that allocate and return via parameter
    int *heap_val = NULL;
    // see alloc_int example below in Functions section

    // function pointers — store a reference to a function
    int (*compare)(const void *, const void *);
    // used heavily with qsort, bsearch, callbacks (see Common Patterns)

    return 0;
}
```

## Memory

C gives you direct control over memory. With that comes the responsibility to free what you allocate — no garbage collector is coming to save you.

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    // stack allocation — automatic, scoped to the block, fast
    int local = 42;  // freed when this function returns
    // stack is small (typically 1-8 MB) — don't put large arrays here

    // heap allocation — manual, persists until you free it
    int *p = malloc(10 * sizeof(int));  // allocates 10 ints, contents are UNINITIALIZED (garbage)
    if (p == NULL) {
        // malloc returns NULL on failure — always check
        perror("malloc");
        return 1;
    }

    // calloc — allocates AND zeros the memory
    int *q = calloc(10, sizeof(int));  // 10 ints, all set to 0
    if (q == NULL) {
        free(p);  // clean up what we already allocated
        return 1;
    }

    // realloc — resize an existing allocation (may move the data)
    int *tmp = realloc(p, 20 * sizeof(int));
    if (tmp == NULL) {
        // realloc failure does NOT free the original — p is still valid
        free(p);
        free(q);
        return 1;
    }
    p = tmp;  // never do p = realloc(p, ...) — if it fails, you leak the original

    // always free what you malloc/calloc/realloc
    free(p);
    p = NULL;  // defensive: prevents use-after-free bugs from silently working

    free(q);
    q = NULL;

    // common pitfalls:

    // 1. use after free — the memory may be reused, you'll read garbage or corrupt data
    //    free(p); printf("%d\n", p[0]);  // undefined behavior

    // 2. double free — freeing the same pointer twice corrupts the allocator's internal state
    //    free(p); free(p);  // undefined behavior, potential security vulnerability

    // 3. memory leak — forgetting to free
    //    int *leak = malloc(100);
    //    leak = malloc(200);  // original 100 bytes are now unreachable

    // 4. stack use after return — returning a pointer to a local variable
    //    int *bad(void) { int x = 42; return &x; }  // x is gone after bad() returns

    // a practical pattern: allocate a struct and its string payload together
    typedef struct {
        size_t len;
        char data[];  // flexible array member (C99) — must be last field
    } Buffer;

    size_t data_len = 64;
    Buffer *buf = malloc(sizeof(Buffer) + data_len);
    if (buf) {
        buf->len = data_len;
        memset(buf->data, 0, data_len);
        free(buf);  // single free for the whole thing
    }

    return 0;
}
```

## Structs

Structs group related data. They're the closest thing C has to objects — but with no methods, no constructors, and no access control.

```c
#include <stdio.h>
#include <stddef.h>  // offsetof

// definition
struct Point {
    double x;
    double y;
};

// typedef saves you from writing "struct" everywhere
typedef struct {
    char name[64];
    int age;
    double salary;
} Employee;

// self-referential structs need the struct tag (common for linked lists, trees)
typedef struct Node {
    int value;
    struct Node *next;  // can't use "Node" here — typedef isn't complete yet
} Node;

// bit fields — pack flags into minimal space (useful for protocols, hardware registers)
typedef struct {
    unsigned int is_active : 1;   // 1 bit
    unsigned int priority  : 3;   // 3 bits (0-7)
    unsigned int category  : 4;   // 4 bits (0-15)
} Flags;

int main(void) {
    // initialization
    struct Point p1 = {1.0, 2.0};              // positional
    struct Point p2 = {.y = 3.0, .x = 4.0};   // designated (C99) — order doesn't matter
    struct Point p3 = {0};                      // zero-initialize all fields

    // access
    p1.x = 10.0;

    // structs are value types — assignment copies all bytes
    struct Point copy = p1;  // independent copy

    // pointer to struct — use -> instead of (*ptr).field
    struct Point *pp = &p1;
    pp->x = 20.0;

    // struct padding — the compiler inserts gaps for alignment
    typedef struct {
        char a;    // 1 byte
        // 3 bytes padding (to align int to 4-byte boundary)
        int b;     // 4 bytes
        char c;    // 1 byte
        // 3 bytes padding (to make struct size a multiple of largest alignment)
    } Padded;
    // sizeof(Padded) is likely 12, not 6
    // reorder fields largest-first to minimize waste:
    typedef struct {
        int b;     // 4 bytes
        char a;    // 1 byte
        char c;    // 1 byte
        // 2 bytes padding
    } Compact;
    // sizeof(Compact) is likely 8

    printf("Padded: %zu, Compact: %zu\n", sizeof(Padded), sizeof(Compact));

    // offsetof macro — gives byte offset of a field within a struct
    printf("offset of salary in Employee: %zu\n", offsetof(Employee, salary));

    // nested structs
    typedef struct {
        struct Point position;
        struct Point velocity;
        double mass;
    } Particle;

    Particle part = {
        .position = {.x = 0, .y = 0},
        .velocity = {.x = 1.5, .y = -0.5},
        .mass = 10.0,
    };

    return 0;
}
```

## Unions and Enums

Unions let multiple fields share the same memory — only one is valid at a time. Enums give names to integer constants.

```c
#include <stdio.h>
#include <string.h>

// a union's size equals its largest member
typedef union {
    int i;
    float f;
    char bytes[4];
} Word;
// sizeof(Word) == 4 (assuming 4-byte int and float)
// writing to .i and reading .f is technically undefined (but widely relied on for type punning)

// enum — named integer constants, starting at 0 by default
typedef enum {
    LOG_DEBUG,    // 0
    LOG_INFO,     // 1
    LOG_WARN,     // 2
    LOG_ERROR,    // 3
} LogLevel;

// explicit values — useful for protocols, file formats
typedef enum {
    HTTP_OK = 200,
    HTTP_NOT_FOUND = 404,
    HTTP_INTERNAL = 500,
} HttpStatus;

// tagged union — the idiomatic way to do variant/sum types in C
typedef enum {
    VAL_INT,
    VAL_FLOAT,
    VAL_STRING,
} ValueType;

typedef struct {
    ValueType type;  // the "tag" — tells you which union member is valid
    union {
        int i;
        double f;
        char str[64];
    } data;
} Value;

void print_value(const Value *v) {
    switch (v->type) {
    case VAL_INT:
        printf("int: %d\n", v->data.i);
        break;
    case VAL_FLOAT:
        printf("float: %f\n", v->data.f);
        break;
    case VAL_STRING:
        printf("string: %s\n", v->data.str);
        break;
    }
}

int main(void) {
    Value v = {.type = VAL_STRING};
    strncpy(v.data.str, "hello", sizeof(v.data.str) - 1);
    print_value(&v);

    // unions for byte-level inspection
    Word w;
    w.i = 0x41424344;
    printf("bytes: %c %c %c %c\n", w.bytes[0], w.bytes[1], w.bytes[2], w.bytes[3]);
    // output depends on endianness — this is how you discover your platform's byte order

    return 0;
}
```

## Control Flow

C's control flow is minimal and predictable. The one surprise: `switch` falls through by default.

```c
#include <stdio.h>

int main(void) {
    int x = 10;

    // if/else — no surprises, but remember: no bool type before C99
    if (x > 0) {
        printf("positive\n");
    } else if (x == 0) {
        printf("zero\n");
    } else {
        printf("negative\n");
    }

    // for loop — C99 allows declaration in the initializer
    for (int i = 0; i < 10; i++) {
        if (i == 3) continue;  // skip to next iteration
        if (i == 7) break;     // exit the loop
        printf("%d ", i);
    }
    printf("\n");

    // while
    int n = 5;
    while (n > 0) {
        n--;
    }

    // do-while — body executes at least once, useful for input validation, retry loops
    do {
        printf("n = %d\n", n);
        n++;
    } while (n < 3);

    // switch — falls through without break (this is a feature, but usually a bug)
    int cmd = 2;
    switch (cmd) {
    case 1:
        printf("create\n");
        break;
    case 2:  // intentional fallthrough
    case 3:
        printf("update or delete\n");
        break;
    default:
        printf("unknown\n");
        break;
    }

    // goto — controversial, but legitimately useful for cleanup in functions with
    // multiple failure points (the Linux kernel uses this pattern everywhere)
    int *buf = NULL;
    FILE *fp = NULL;

    buf = malloc(256);
    if (!buf) goto cleanup;

    fp = fopen("/tmp/test.txt", "r");
    if (!fp) goto cleanup;

    // ... do work ...

cleanup:
    // single cleanup path — no duplicated free/fclose calls
    if (fp) fclose(fp);
    free(buf);  // free(NULL) is safe and does nothing

    return 0;
}
```

## Functions

Functions are pass-by-value only. To modify the caller's data, pass a pointer.

```c
#include <stdio.h>
#include <stdlib.h>

// forward declaration (prototype) — tells the compiler the signature before the definition
int add(int a, int b);

// definition
int add(int a, int b) {
    return a + b;
}

// pass by value — the function gets a copy
void try_modify(int x) {
    x = 999;  // modifies the local copy only
}

// pass by pointer — the function gets the address, can modify the original
void swap(int *a, int *b) {
    int tmp = *a;
    *a = *b;
    *b = tmp;
}

// output parameter pattern — common for functions that can fail
// returns 0 on success, -1 on failure. result goes into *out.
int safe_divide(int a, int b, int *out) {
    if (b == 0) return -1;
    *out = a / b;
    return 0;
}

// double pointer — when the function needs to allocate and return a pointer
int alloc_buffer(char **out, size_t size) {
    *out = malloc(size);
    if (*out == NULL) return -1;
    return 0;
}

// static functions — file-scoped, not visible to other translation units
// this is C's version of "private" — use it by default, expose only what's needed
static int helper(int x) {
    return x * 2;
}

// inline hint — suggests the compiler inline the function (C99+)
// the compiler can ignore this, and often makes better inlining decisions on its own
static inline int max(int a, int b) {
    return (a > b) ? a : b;
}

// variadic functions — printf is the classic example
// requires <stdarg.h>, hard to use safely (no type checking on the varargs)
#include <stdarg.h>
int sum(int count, ...) {
    va_list args;
    va_start(args, count);

    int total = 0;
    for (int i = 0; i < count; i++) {
        total += va_arg(args, int);  // caller must pass the right types — no enforcement
    }

    va_end(args);
    return total;
}

int main(void) {
    int a = 5, b = 10;
    swap(&a, &b);  // pass addresses
    printf("a=%d, b=%d\n", a, b);  // a=10, b=5

    int result;
    if (safe_divide(10, 3, &result) == 0) {
        printf("10/3 = %d\n", result);
    }

    char *buf = NULL;
    if (alloc_buffer(&buf, 128) == 0) {
        // use buf...
        free(buf);
    }

    printf("sum: %d\n", sum(4, 10, 20, 30, 40));  // 100

    return 0;
}
```

## Preprocessor

The preprocessor runs before compilation — it operates on text, not C syntax. Powerful but error-prone.

```c
// include guards — prevent double inclusion (every header needs this)
#ifndef MY_HEADER_H
#define MY_HEADER_H
// ... header contents ...
#endif
// alternatively: #pragma once (non-standard but universally supported)

#include <stdio.h>

// simple constants — prefer this over magic numbers
#define MAX_BUFFER_SIZE 4096
#define PI 3.14159265358979

// macros with arguments — use parentheses around EVERYTHING to avoid precedence bugs
#define MIN(a, b) ((a) < (b) ? (a) : (b))
#define SQUARE(x) ((x) * (x))
// SQUARE(i++) expands to ((i++) * (i++)) — increments twice! this is why macros are dangerous

// multi-line macros — use do { } while(0) so they work in all contexts
#define LOG_ERROR(msg) do { \
    fprintf(stderr, "[ERROR] %s:%d: %s\n", __FILE__, __LINE__, msg); \
} while (0)
// the do-while trick makes this safe after if/else without braces

// stringification and token pasting
#define STRINGIFY(x) #x          // turns the argument into a string literal
#define CONCAT(a, b) a##b        // joins two tokens: CONCAT(foo, _bar) → foo_bar
#define ASSERT(expr) do { \
    if (!(expr)) { \
        fprintf(stderr, "assertion failed: %s at %s:%d\n", \
                STRINGIFY(expr), __FILE__, __LINE__); \
        abort(); \
    } \
} while (0)

// conditional compilation — essential for platform-specific code
#ifdef _WIN32
    // windows-specific code
#elif defined(__linux__)
    // linux-specific code
#elif defined(__APPLE__)
    // macOS-specific code
#else
    #error "unsupported platform"
#endif

// feature detection
#if __STDC_VERSION__ >= 201112L
    // C11 features available
    #include <stdnoreturn.h>
#endif

// X-macros — generate repetitive code from a single definition
// define the list once, expand it differently in each context
#define HTTP_METHODS(X) \
    X(GET, "GET")       \
    X(POST, "POST")     \
    X(PUT, "PUT")       \
    X(DELETE, "DELETE")

// generate an enum
typedef enum {
    #define X_ENUM(name, str) METHOD_##name,
    HTTP_METHODS(X_ENUM)
    #undef X_ENUM
    METHOD_COUNT
} HttpMethod;

// generate a string lookup table
static const char *method_names[] = {
    #define X_STR(name, str) [METHOD_##name] = str,
    HTTP_METHODS(X_STR)
    #undef X_STR
};

// predefined macros
// __FILE__     — current filename (string)
// __LINE__     — current line number (int)
// __func__     — current function name (C99, string)
// __DATE__     — compilation date
// __STDC__     — 1 if conforming compiler

int main(void) {
    int a = 3, b = 7;
    printf("min: %d\n", MIN(a, b));

    LOG_ERROR("something went wrong");

    printf("method 0: %s\n", method_names[METHOD_GET]);

    return 0;
}
```

## File I/O

File I/O in C uses opaque `FILE *` handles. Always check return values — files can fail to open, reads can be short, disks can fill up.

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    // text mode — newline translation happens on Windows (\r\n <-> \n)
    FILE *fp = fopen("example.txt", "w");  // "w" = write (truncates), "a" = append
    if (fp == NULL) {
        perror("fopen");  // prints "fopen: No such file or directory" (or whatever errno says)
        return 1;
    }

    fprintf(fp, "line one: %d\n", 42);  // printf-style formatting to a file
    fputs("line two\n", fp);            // write a string (no formatting)
    fclose(fp);

    // reading text
    fp = fopen("example.txt", "r");
    if (!fp) { perror("fopen"); return 1; }

    char line[256];
    while (fgets(line, sizeof(line), fp) != NULL) {
        // fgets includes the trailing \n — strip it if you don't want it
        line[strcspn(line, "\n")] = '\0';
        printf("read: [%s]\n", line);
    }
    fclose(fp);

    // binary mode — no newline translation, use for anything non-text
    // "rb" = read binary, "wb" = write binary
    fp = fopen("data.bin", "wb");
    if (!fp) { perror("fopen"); return 1; }

    int32_t numbers[] = {1, 2, 3, 4, 5};
    size_t written = fwrite(numbers, sizeof(int32_t), 5, fp);
    // fwrite returns the number of ELEMENTS written — check it
    if (written != 5) {
        perror("fwrite");
    }
    fclose(fp);

    // reading binary
    fp = fopen("data.bin", "rb");
    if (!fp) { perror("fopen"); return 1; }

    int32_t readback[5];
    size_t count = fread(readback, sizeof(int32_t), 5, fp);
    if (count != 5) {
        if (feof(fp)) {
            printf("unexpected end of file\n");
        } else if (ferror(fp)) {
            perror("fread");
        }
    }
    fclose(fp);

    // seeking — random access within a file
    fp = fopen("data.bin", "rb");
    if (!fp) { perror("fopen"); return 1; }

    fseek(fp, 2 * sizeof(int32_t), SEEK_SET);  // jump to the 3rd element
    int32_t third;
    fread(&third, sizeof(int32_t), 1, fp);
    printf("third: %d\n", third);  // 3

    // get file size
    fseek(fp, 0, SEEK_END);
    long size = ftell(fp);
    printf("file size: %ld bytes\n", size);

    fclose(fp);

    return 0;
}
```

## Error Handling

C has no exceptions. Functions signal errors through return values and the global `errno`. Consistent error handling is on you.

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>    // errno, ENOENT, ENOMEM, etc.
#include <setjmp.h>   // setjmp, longjmp (rarely needed)

// convention: return 0 for success, -1 (or negative) for failure, set errno
int read_file(const char *path, char *buf, size_t buf_size) {
    FILE *fp = fopen(path, "r");
    if (!fp) {
        // errno is already set by fopen — the caller can inspect it
        return -1;
    }

    size_t n = fread(buf, 1, buf_size - 1, fp);
    if (ferror(fp)) {
        fclose(fp);
        return -1;
    }
    buf[n] = '\0';
    fclose(fp);
    return 0;
}

// alternative: return a result struct (avoids the errno global state)
typedef struct {
    int ok;           // 1 = success, 0 = failure
    int error_code;
    char error_msg[128];
} Result;

Result do_something(void) {
    Result r = {0};
    FILE *fp = fopen("/nonexistent", "r");
    if (!fp) {
        r.error_code = errno;
        snprintf(r.error_msg, sizeof(r.error_msg), "open failed: %s", strerror(errno));
        return r;
    }
    fclose(fp);
    r.ok = 1;
    return r;
}

// setjmp/longjmp — C's version of exceptions. use sparingly.
// mostly useful for jumping out of deeply nested calls on fatal errors.
static jmp_buf jump_target;

void risky_operation(void) {
    // longjmp unwinds the stack back to where setjmp was called
    // WARNING: does NOT call destructors, does NOT free heap memory, does NOT close files
    longjmp(jump_target, 1);  // second arg becomes setjmp's return value
}

int main(void) {
    // errno approach
    char buf[1024];
    if (read_file("/tmp/no_such_file.txt", buf, sizeof(buf)) != 0) {
        // perror prints: "read_file: No such file or directory"
        perror("read_file");

        // or build your own message
        fprintf(stderr, "error %d: %s\n", errno, strerror(errno));
    }

    // result struct approach
    Result r = do_something();
    if (!r.ok) {
        fprintf(stderr, "%s\n", r.error_msg);
    }

    // setjmp/longjmp
    if (setjmp(jump_target) != 0) {
        // landed here from longjmp — handle the error
        fprintf(stderr, "recovered from longjmp\n");
        return 1;
    }
    risky_operation();  // this will longjmp back

    return 0;
}
```

## Common Patterns

Patterns that show up constantly in production C code — API design, callbacks, and data structure techniques.

```c
#include <stdio.h>
#include <stdlib.h>
#include <stddef.h>
#include <string.h>

// --- opaque types (information hiding) ---
// the public header exposes only a forward declaration — callers can't see the struct fields
// this is C's version of encapsulation. libraries like sqlite3 use this everywhere.

// in db.h:
typedef struct Database Database;  // opaque — no field access from outside
Database *db_open(const char *path);
void db_close(Database *db);

// in db.c (private implementation):
struct Database {
    FILE *fp;
    int transaction_count;
    // callers never see these fields — you can change them without breaking ABI
};

Database *db_open(const char *path) {
    Database *db = calloc(1, sizeof(*db));  // sizeof(*db) adapts if the struct changes
    if (!db) return NULL;
    db->fp = fopen(path, "r+");
    if (!db->fp) { free(db); return NULL; }
    return db;
}

void db_close(Database *db) {
    if (!db) return;
    if (db->fp) fclose(db->fp);
    free(db);
}

// --- callbacks ---
// function pointers enable generic algorithms and plugin architectures

typedef int (*Comparator)(const void *, const void *);

int int_compare(const void *a, const void *b) {
    int ia = *(const int *)a;
    int ib = *(const int *)b;
    // avoid "return ia - ib" — it overflows for extreme values
    return (ia > ib) - (ia < ib);
}

// --- container_of macro ---
// given a pointer to a struct member, recover the pointer to the enclosing struct.
// the Linux kernel uses this for its linked lists, rbtrees, hash tables.
#define container_of(ptr, type, member) \
    ((type *)((char *)(ptr) - offsetof(type, member)))

// intrusive linked list — the node is embedded inside your data struct
typedef struct ListNode {
    struct ListNode *next;
    struct ListNode *prev;
} ListNode;

typedef struct {
    int id;
    char name[32];
    ListNode node;  // embedded, not a separate allocation
} User;

// recover the User from a ListNode pointer
void process_node(ListNode *n) {
    User *u = container_of(n, User, node);
    printf("user: %d %s\n", u->id, u->name);
}

// --- flexible array member (C99) ---
// allocate a struct and a variable-length payload in a single allocation
typedef struct {
    size_t len;
    char data[];  // must be the last field — occupies zero bytes in sizeof
} Message;

Message *message_new(const char *text) {
    size_t len = strlen(text);
    Message *m = malloc(sizeof(Message) + len + 1);
    if (!m) return NULL;
    m->len = len;
    memcpy(m->data, text, len + 1);  // include null terminator
    return m;
}

// --- cleanup with goto ---
// the standard pattern for functions with multiple resources to acquire
int process_file(const char *input_path, const char *output_path) {
    int ret = -1;
    FILE *in = NULL;
    FILE *out = NULL;
    char *buf = NULL;

    in = fopen(input_path, "r");
    if (!in) goto done;

    out = fopen(output_path, "w");
    if (!out) goto done;

    buf = malloc(4096);
    if (!buf) goto done;

    // ... do work with in, out, buf ...
    ret = 0;  // success

done:
    free(buf);
    if (out) fclose(out);
    if (in) fclose(in);
    return ret;
}

int main(void) {
    // qsort with callback
    int nums[] = {5, 2, 8, 1, 9, 3};
    size_t count = sizeof(nums) / sizeof(nums[0]);
    qsort(nums, count, sizeof(int), int_compare);

    for (size_t i = 0; i < count; i++) {
        printf("%d ", nums[i]);
    }
    printf("\n");

    // flexible array member
    Message *msg = message_new("hello from C");
    if (msg) {
        printf("message (%zu): %s\n", msg->len, msg->data);
        free(msg);
    }

    // opaque type
    Database *db = db_open("/tmp/test.db");
    db_close(db);

    return 0;
}
```
