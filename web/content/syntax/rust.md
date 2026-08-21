## Hello World

Every Rust program starts at `fn main`. Rust uses `!` to denote macros, which are compile-time code generators — not regular function calls.

```rust
fn main() {
    // println! is a macro, not a function. the ! matters.
    // macros can accept variable argument counts — functions can't in rust.
    println!("hello, world");

    // format strings work like printf but with {} placeholders
    let name = "lighthouse";
    println!("welcome to {name}"); // inline variable (rust 1.58+)
    println!("welcome to {}", name); // positional argument

    // eprintln! writes to stderr — use it for diagnostics
    eprintln!("this goes to stderr");

    // dbg! prints the expression AND its value, returns the value.
    // invaluable for debugging — it shows file:line too.
    let x = dbg!(2 + 3); // prints [src/main.rs:14] 2 + 3 = 5
    println!("{x}");
}
```

## Variables

Rust variables are immutable by default. This is a deliberate design choice — it forces you to be explicit about what can change, which prevents entire categories of bugs.

```rust
fn main() {
    // immutable by default — this won't compile if you try to reassign
    let x = 5;

    // mut opts into mutability — the compiler tracks this
    let mut y = 10;
    y += 1;

    // type annotations — usually unnecessary because rust infers types,
    // but required when the compiler can't figure it out
    let z: i64 = 100_000;

    // underscores in numbers are visual separators, ignored by compiler
    let big = 1_000_000;

    // shadowing: you can re-declare a variable with the same name.
    // this creates a NEW binding — it's not mutation.
    // useful for transforming a value while keeping the name.
    let input = "42";
    let input: i32 = input.parse().unwrap(); // same name, different type

    // const must have a type annotation, evaluated at compile time.
    // convention: SCREAMING_SNAKE_CASE
    const MAX_CONNECTIONS: u32 = 1024;

    // static is like const but has a fixed memory address.
    // rarely needed unless you're doing FFI or need a global reference.
    static VERSION: &str = "0.1.0";
}
```

## Types

Rust is statically typed with no implicit conversions. If you want to go from i32 to i64, you say so explicitly. This catches overflow and truncation bugs at compile time.

```rust
fn main() {
    // signed integers: i8, i16, i32, i64, i128
    // unsigned integers: u8, u16, u32, u64, u128
    let a: i32 = -42;
    let b: u8 = 255; // max value for u8

    // isize and usize match pointer width (64-bit on modern systems).
    // usize is what you use for indexing into collections.
    let index: usize = 0;

    // floats: f32 and f64. default inference is f64.
    let pi: f64 = 3.14159;
    let approx: f32 = 3.14;

    // bool: true or false, 1 byte
    let active: bool = true;

    // char: a unicode scalar value, 4 bytes (not 1 byte like C)
    let emoji: char = '🦀';
    let letter: char = 'a';

    // tuple: fixed-size, mixed types. access by index with .N
    let point: (f64, f64, f64) = (1.0, 2.0, 3.0);
    let x = point.0;
    let y = point.1;

    // destructure a tuple into named bindings
    let (r, g, b) = (255, 128, 0);

    // array: fixed-size, same type. stack-allocated.
    // the type is [T; N] where N is part of the type — [i32; 3] != [i32; 4]
    let rgb: [u8; 3] = [255, 0, 128];
    let zeros = [0u8; 1024]; // 1024 zeros

    // explicit casting — rust never does this implicitly
    let small: i32 = 42;
    let big: i64 = small as i64;

    // unit type () — like void but it's an actual value.
    // functions that return nothing actually return ()
    let nothing: () = ();
}
```

## Strings

Rust has two main string types because of its ownership model. `String` is an owned, heap-allocated, growable buffer. `&str` is a borrowed reference to string data (a "string slice"). This distinction matters for memory safety.

```rust
fn main() {
    // &str: a reference to a string slice. string literals are &'static str
    // — they live in the binary and last the entire program.
    let greeting: &str = "hello";

    // String: owned, heap-allocated, growable. you need this when
    // you want to build or modify strings at runtime.
    let mut owned = String::from("hello");
    owned.push_str(", world"); // append a &str
    owned.push('!'); // append a single char

    // converting between the two
    let s1: String = "hello".to_string(); // &str -> String
    let s2: String = String::from("hello"); // same thing
    let s3: &str = &owned; // String -> &str (deref coercion)

    // format! builds a String without printing it
    let name = "rust";
    let msg = format!("learning {name} in {}", 2026);

    // strings are UTF-8. len() returns bytes, not characters.
    let emoji = "🦀";
    println!("{}", emoji.len()); // 4 bytes
    println!("{}", emoji.chars().count()); // 1 character

    // you cannot index strings by position (s[0]) because UTF-8
    // characters are variable-width. use .chars() or .bytes() instead.
    for ch in "hello".chars() {
        print!("{ch} ");
    }

    // slicing works on byte boundaries — panics if you split a multi-byte char
    let hello = "hello world";
    let word = &hello[0..5]; // "hello"

    // common string operations
    let trimmed = "  spaces  ".trim();
    let upper = "hello".to_uppercase();
    let parts: Vec<&str> = "a,b,c".split(',').collect();
    let joined = parts.join("-"); // "a-b-c"

    // String is really Vec<u8> with a UTF-8 guarantee.
    // this is why you can't just index into it.
    let bytes: &[u8] = owned.as_bytes();
}
```

## Ownership

This is the core concept that makes Rust unique. Every value has exactly one owner. When the owner goes out of scope, the value is dropped (freed). This eliminates use-after-free, double-free, and data races — without a garbage collector.

```rust
fn main() {
    // when you assign a String (heap data) to another variable,
    // ownership MOVES. the original variable becomes invalid.
    let s1 = String::from("hello");
    let s2 = s1; // s1 is moved to s2
    // println!("{s1}"); // COMPILE ERROR: s1 no longer valid

    // if you need both, clone explicitly. this copies heap data.
    let s3 = s2.clone();
    println!("{s2} {s3}"); // both valid

    // primitive types (i32, f64, bool, char) implement the Copy trait.
    // they're stack-only, so "moving" them just copies the bits.
    let a = 42;
    let b = a; // copy, not move — a is still valid
    println!("{a} {b}");

    // passing a String to a function moves it — you lose access
    let name = String::from("rust");
    takes_ownership(name);
    // println!("{name}"); // COMPILE ERROR: name was moved

    // borrowing: lend a reference without giving up ownership.
    // &T is a shared (immutable) reference.
    let data = String::from("borrow me");
    print_length(&data); // data is borrowed, not moved
    println!("{data}"); // still valid

    // &mut T is an exclusive (mutable) reference.
    // rust enforces: either ONE &mut OR any number of & — never both.
    // this rule prevents data races at compile time.
    let mut buffer = String::from("hello");
    append_world(&mut buffer);
    println!("{buffer}"); // "hello world"
}

fn takes_ownership(s: String) {
    println!("{s}");
} // s is dropped here, memory freed

fn print_length(s: &String) {
    // s is a reference — we can read but not modify
    println!("length: {}", s.len());
} // s goes out of scope but doesn't own the data, so nothing is dropped

fn append_world(s: &mut String) {
    s.push_str(" world");
}

// lifetimes: when returning references, the compiler needs to know
// how long the returned reference is valid. lifetimes are annotations
// that describe these relationships.
// 'a says: the returned reference lives at least as long as both inputs.
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}
```

## Structs

Structs are Rust's primary way to define custom types. Unlike classes in OOP languages, data (fields) and behavior (methods) are defined separately.

```rust
// derive macros auto-generate trait implementations.
// Debug lets you print with {:?}, Clone lets you .clone()
#[derive(Debug, Clone)]
struct Server {
    host: String,
    port: u16,
    max_connections: usize,
}

// impl block: where you define methods and associated functions
impl Server {
    // associated function (no self) — like a static method or constructor.
    // convention: new() for the primary constructor.
    fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            max_connections: 1024,
        }
    }

    // method: takes &self (immutable borrow of the instance)
    fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    // mutable method: takes &mut self
    fn set_max_connections(&mut self, max: usize) {
        self.max_connections = max;
    }

    // consuming method: takes self (ownership). the instance is
    // moved into this method and can't be used after.
    // useful for builder patterns or state transitions.
    fn into_address(self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// tuple structs: named types wrapping other types.
// useful for type safety — Port(8080) and Count(8080) are different types.
struct Port(u16);
struct Milliseconds(u64);

// unit struct: no fields. used as markers or for trait implementations.
struct Production;

fn main() {
    let mut server = Server::new("0.0.0.0", 8080);
    println!("{}", server.address());
    server.set_max_connections(2048);

    // struct update syntax: create a new instance, copying fields from another.
    // ..server moves any non-Copy fields from server.
    let backup = Server {
        port: 8081,
        ..server.clone()
    };

    // destructuring
    let Server { host, port, .. } = &backup;
    println!("backup at {host}:{port}");

    // Debug printing (from #[derive(Debug)])
    println!("{:?}", backup);
    println!("{:#?}", backup); // pretty-printed
}
```

## Enums

Enums in Rust are algebraic data types — each variant can hold different data. Combined with pattern matching, they replace null checks, tagged unions, and exception handling from other languages.

```rust
// variants can hold no data, named fields, or positional fields
#[derive(Debug)]
enum Command {
    Quit,
    Echo(String),
    Move { x: i32, y: i32 },
    Color(u8, u8, u8),
}

impl Command {
    fn execute(&self) {
        // match must be exhaustive — you handle every variant or the
        // compiler rejects your code. no forgotten cases.
        match self {
            Command::Quit => println!("quitting"),
            Command::Echo(msg) => println!("{msg}"),
            Command::Move { x, y } => println!("moving to ({x}, {y})"),
            Command::Color(r, g, b) => println!("rgb({r}, {g}, {b})"),
        }
    }
}

fn main() {
    let cmd = Command::Move { x: 10, y: 20 };
    cmd.execute();

    // Option<T>: rust's replacement for null. a value is either
    // Some(T) or None. you MUST handle the None case.
    let maybe: Option<i32> = Some(42);
    let nothing: Option<i32> = None;

    // unwrap_or provides a default when None
    let val = nothing.unwrap_or(0);

    // map transforms the inner value if present
    let doubled = maybe.map(|x| x * 2); // Some(84)

    // Result<T, E>: for operations that can fail.
    // Ok(T) on success, Err(E) on failure.
    let parsed: Result<i32, _> = "42".parse();
    let failed: Result<i32, _> = "not_a_number".parse();

    match parsed {
        Ok(n) => println!("parsed: {n}"),
        Err(e) => println!("error: {e}"),
    }
}
```

## Pattern Matching

Pattern matching in Rust goes beyond switch statements. It destructures data, binds variables, and the compiler guarantees every case is handled.

```rust
#[derive(Debug)]
enum Status {
    Running { pid: u32 },
    Stopped,
    Failed(String),
}

fn main() {
    let status = Status::Running { pid: 1234 };

    // match: must cover all variants
    match &status {
        Status::Running { pid } => println!("process {pid} running"),
        Status::Stopped => println!("stopped"),
        Status::Failed(reason) => println!("failed: {reason}"),
    }

    // match with guards: add conditions to patterns
    let code: i32 = 404;
    let message = match code {
        200 => "ok",
        301 | 302 => "redirect", // multiple values with |
        c if (400..500).contains(&c) => "client error", // guard
        c if (500..600).contains(&c) => "server error",
        _ => "unknown", // _ is the wildcard / catch-all
    };
    println!("{code}: {message}");

    // if let: when you only care about one variant.
    // less verbose than a full match when you'd just ignore the rest.
    if let Status::Running { pid } = &status {
        println!("still running with pid {pid}");
    }

    // while let: loop as long as a pattern matches.
    // commonly used to drain iterators or channels.
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("popped: {top}");
    }

    // destructuring tuples
    let point = (3, 7);
    let (x, y) = point;

    // nested destructuring
    let ((a, b), c) = ((1, 2), 3);

    // let-else: destructure or bail out. introduced in rust 1.65.
    // the else block must diverge (return, break, continue, panic).
    let value: Option<i32> = Some(42);
    let Some(inner) = value else {
        println!("was None");
        return;
    };
    println!("got {inner}");

    // matching references
    let reference = &42;
    match reference {
        &val => println!("got value: {val}"),
    }

    // @ bindings: capture the matched value while also testing a pattern
    let num = 15;
    match num {
        n @ 1..=12 => println!("month {n}"),
        n @ 13..=24 => println!("month+12: {n}"),
        n => println!("out of range: {n}"),
    }
}
```

## Collections

The standard library provides growable, heap-allocated collections. `Vec` and `HashMap` cover most use cases.

```rust
use std::collections::HashMap;

fn main() {
    // Vec<T>: growable array. the most common collection.
    let mut numbers: Vec<i32> = Vec::new();
    numbers.push(1);
    numbers.push(2);
    numbers.push(3);

    // vec! macro for initialization
    let mut names = vec!["alice", "bob", "charlie"];

    // indexing panics on out-of-bounds. .get() returns Option<&T>.
    let first = numbers[0]; // 1, panics if empty
    let maybe = numbers.get(99); // None, no panic

    // iterators: rust's preferred way to process collections.
    // .iter() borrows, .into_iter() consumes, .iter_mut() borrows mutably.

    // map + collect: transform and gather into a new collection
    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

    // filter: keep elements matching a predicate
    let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();

    // chaining is idiomatic — reads top-to-bottom like a pipeline
    let result: Vec<String> = names
        .iter()
        .filter(|name| name.len() > 3)
        .map(|name| name.to_uppercase())
        .collect();

    // fold: accumulate a result (like reduce in other languages)
    let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);
    // .sum() is a shortcut when the types work out
    let sum2: i32 = numbers.iter().sum();

    // find: returns the first match as Option<&T>
    let found = numbers.iter().find(|&&x| x > 1);

    // any/all: boolean checks
    let has_negative = numbers.iter().any(|x| *x < 0);
    let all_positive = numbers.iter().all(|x| *x > 0);

    // enumerate: get index + value pairs
    for (i, name) in names.iter().enumerate() {
        println!("{i}: {name}");
    }

    // HashMap<K, V>
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert("alice".to_string(), 100);
    scores.insert("bob".to_string(), 85);

    // entry API: insert-if-absent or modify-in-place.
    // avoids double lookups that plague most map usage.
    scores.entry("charlie".to_string()).or_insert(0);

    // modify existing value
    let alice_score = scores.entry("alice".to_string()).or_insert(0);
    *alice_score += 10;

    // iterate over key-value pairs
    for (name, score) in &scores {
        println!("{name}: {score}");
    }

    // collect pairs into a HashMap
    let pairs = vec![("x", 1), ("y", 2)];
    let map: HashMap<&str, i32> = pairs.into_iter().collect();

    // retain: in-place filter
    let mut vals = vec![1, 2, 3, 4, 5];
    vals.retain(|x| x % 2 == 0); // [2, 4]

    // dedup: remove consecutive duplicates (sort first for full dedup)
    let mut duped = vec![1, 1, 2, 3, 3, 3];
    duped.dedup(); // [1, 2, 3]
}
```

## Error Handling

Rust has no exceptions. Errors are values — you return them, match on them, and propagate them with `?`. This makes error paths explicit and impossible to accidentally ignore.

```rust
use std::fmt;
use std::fs;
use std::io;
use std::num::ParseIntError;

// the ? operator: if the Result is Ok, unwrap it. if Err, return
// the error early from the current function. this is the idiomatic
// way to propagate errors — replaces try/catch without the hidden control flow.
fn read_config(path: &str) -> Result<String, io::Error> {
    let content = fs::read_to_string(path)?; // returns Err early if file not found
    Ok(content)
}

// custom error type: when your function can fail in multiple ways
#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
    Validation(String),
}

// Display is required for error types — it's the human-readable message
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "io error: {e}"),
            AppError::Parse(e) => write!(f, "parse error: {e}"),
            AppError::Validation(msg) => write!(f, "validation: {msg}"),
        }
    }
}

// From<T> lets ? auto-convert errors. without this, you'd need
// .map_err() on every call that returns a different error type.
impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

// now ? works seamlessly across io::Error and ParseIntError
fn parse_port_from_file(path: &str) -> Result<u16, AppError> {
    let content = fs::read_to_string(path)?; // io::Error -> AppError via From
    let port: u16 = content.trim().parse()?; // ParseIntError -> AppError via From

    if port < 1024 {
        return Err(AppError::Validation("port must be >= 1024".into()));
    }

    Ok(port)
}

fn main() {
    // match on Result for explicit handling
    match parse_port_from_file("port.txt") {
        Ok(port) => println!("port: {port}"),
        Err(e) => eprintln!("error: {e}"),
    }

    // unwrap: extract Ok value, panic on Err. acceptable in tests and
    // prototypes. in production code, handle the error or use expect().
    // let val = risky_call().unwrap();

    // expect: like unwrap but with a custom panic message
    // let val = risky_call().expect("config file must exist");

    // unwrap_or / unwrap_or_else: provide a fallback
    let port = "8080".parse::<u16>().unwrap_or(3000);
    let port2 = "bad".parse::<u16>().unwrap_or_else(|_| {
        eprintln!("using default port");
        3000
    });

    // in real projects, use thiserror for library errors (derive macro
    // that generates Display and From impls) or anyhow for applications
    // (erases error types, adds context). these replace the manual
    // boilerplate above.
    //
    // thiserror example:
    //   #[derive(thiserror::Error, Debug)]
    //   enum AppError {
    //       #[error("io error: {0}")]
    //       Io(#[from] io::Error),
    //       #[error("parse error: {0}")]
    //       Parse(#[from] ParseIntError),
    //   }
    //
    // anyhow example:
    //   fn main() -> anyhow::Result<()> {
    //       let content = fs::read_to_string("config.txt")
    //           .context("failed to read config")?;
    //       Ok(())
    //   }
}
```

## Traits

Traits define shared behavior — similar to interfaces in Go or typeclasses in Haskell. They're Rust's primary abstraction mechanism and the foundation of generics.

```rust
use std::fmt;

// trait definition: a set of methods a type must implement
trait Summary {
    // required method: implementors must define this
    fn summarize(&self) -> String;

    // default method: implementors get this for free but can override.
    // default methods can call other trait methods.
    fn preview(&self) -> String {
        format!("{}...", &self.summarize()[..50.min(self.summarize().len())])
    }
}

struct Article {
    title: String,
    content: String,
}

struct Tweet {
    username: String,
    body: String,
}

// implement a trait for a type
impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}", self.title, &self.content[..100.min(self.content.len())])
    }
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("@{}: {}", self.username, self.body)
    }
}

// trait bounds: accept any type that implements Summary.
// this is monomorphized — the compiler generates a separate
// version of this function for each concrete type used.
fn print_summary(item: &impl Summary) {
    println!("{}", item.summarize());
}

// equivalent syntax using generics (more flexible for complex bounds)
fn print_summary_verbose<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}

// multiple trait bounds
fn print_and_debug(item: &(impl Summary + fmt::Debug)) {
    println!("{:?}", item);
    println!("{}", item.summarize());
}

// dyn Trait: dynamic dispatch via a vtable pointer.
// use when you need a collection of different types implementing the same trait.
// costs a pointer indirection at runtime (vs static dispatch which is zero-cost).
fn get_summaries(items: &[Box<dyn Summary>]) -> Vec<String> {
    items.iter().map(|item| item.summarize()).collect()
}

// returning trait objects: when the concrete type varies
fn make_summarizable(is_tweet: bool) -> Box<dyn Summary> {
    if is_tweet {
        Box::new(Tweet {
            username: "rust".to_string(),
            body: "hello from rust".to_string(),
        })
    } else {
        Box::new(Article {
            title: "Rust".to_string(),
            content: "Rust is great".to_string(),
        })
    }
}

// impl Trait in return position: when there's only ONE concrete type
// (the compiler must be able to determine the type at compile time)
fn make_article() -> impl Summary {
    Article {
        title: "Title".to_string(),
        content: "Content".to_string(),
    }
}

fn main() {
    let article = Article {
        title: "Ownership in Rust".to_string(),
        content: "The ownership system is...".to_string(),
    };
    print_summary(&article);
}
```

## Generics

Generics let you write code that works with multiple types. Rust compiles generic code into specialized versions for each type used (monomorphization), so there's zero runtime overhead compared to writing the same code by hand.

```rust
// generic function: works for any type that can be compared
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut biggest = &list[0];
    for item in &list[1..] {
        if item > biggest {
            biggest = item;
        }
    }
    biggest
}

// generic struct
#[derive(Debug)]
struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Self {
        Self { first, second }
    }
}

// conditional implementation: only implement for types that satisfy bounds.
// this method only exists when T is printable.
impl<T: std::fmt::Display, U: std::fmt::Display> Pair<T, U> {
    fn display(&self) {
        println!("({}, {})", self.first, self.second);
    }
}

// where clauses: cleaner syntax when bounds get complex
fn process<T, U>(t: T, u: U) -> String
where
    T: std::fmt::Display + Clone,
    U: std::fmt::Debug + Into<String>,
{
    format!("{t} - {:?}", u)
}

// phantom type pattern: use generics to add compile-time type safety
// without storing the generic type. the zero-size PhantomData marker
// tells the compiler to act as if the struct contains a T.
use std::marker::PhantomData;

struct Meters;
struct Seconds;

struct Measurement<Unit> {
    value: f64,
    _unit: PhantomData<Unit>,
}

impl<Unit> Measurement<Unit> {
    fn new(value: f64) -> Self {
        Self {
            value,
            _unit: PhantomData,
        }
    }
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    println!("largest: {}", largest(&numbers));

    let pair = Pair::new("hello", 42);
    pair.display();

    // these are different types — can't accidentally add meters to seconds
    let distance = Measurement::<Meters>::new(100.0);
    let duration = Measurement::<Seconds>::new(9.58);
}
```

## Closures

Closures are anonymous functions that capture their environment. They're used heavily with iterators, thread spawning, and callbacks. The compiler infers which capture mode to use (borrow, mutable borrow, or move).

```rust
fn main() {
    // basic closure syntax — type inference handles parameter and return types
    let add = |a, b| a + b;
    let sum = add(2, 3);

    // explicit types if needed
    let add_typed = |a: i32, b: i32| -> i32 { a + b };

    // closures capture variables from enclosing scope.
    // the compiler figures out the least restrictive capture mode.
    let prefix = String::from("hello");

    // captures &prefix (immutable borrow) — Fn trait
    let greet = |name: &str| format!("{prefix} {name}");
    println!("{}", greet("world"));
    println!("{prefix}"); // still usable because greet only borrows it

    // captures &mut — FnMut trait
    let mut count = 0;
    let mut increment = || {
        count += 1; // mutably borrows count
        count
    };
    println!("{}", increment());
    println!("{}", increment());

    // move keyword: force the closure to take ownership of captured values.
    // essential when the closure outlives the current scope (e.g., threads).
    let name = String::from("rust");
    let greeting = move || {
        println!("hello {name}"); // name is moved into the closure
    };
    // println!("{name}"); // COMPILE ERROR: name was moved
    greeting();

    // closures as function parameters
    let numbers = vec![1, 2, 3, 4, 5];

    // Fn: borrows captures immutably (can call multiple times)
    let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();

    // FnOnce: consumes captures (can only call once).
    // used when the closure gives away ownership of something.
    let data = vec![1, 2, 3];
    let consume = move || {
        drop(data); // data is consumed
    };
    consume();
    // consume(); // COMPILE ERROR: already called (FnOnce)

    // returning closures: must use impl Fn or Box<dyn Fn>
    fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
        move |y| x + y
    }
    let add_five = make_adder(5);
    println!("{}", add_five(3)); // 8
}

// accepting closures in function signatures
fn apply<F>(value: i32, f: F) -> i32
where
    F: Fn(i32) -> i32,
{
    f(value)
}
```

## Concurrency

Rust's ownership system prevents data races at compile time. The `Send` and `Sync` marker traits tell the compiler which types are safe to share across threads — if your code compiles, it's free of data races.

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

fn main() {
    // basic thread: spawn takes a closure. returns a JoinHandle
    // to wait for the thread to finish.
    let handle = thread::spawn(|| {
        println!("hello from a thread");
        42
    });
    let result = handle.join().unwrap(); // blocks until thread finishes

    // move is usually required because the thread may outlive
    // the scope that spawned it. the compiler enforces this.
    let data = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        println!("data: {data:?}");
    });
    handle.join().unwrap();

    // sharing data across threads requires Arc (atomic reference counting).
    // Rc is NOT thread-safe — Arc adds atomic operations.
    // for mutable access, wrap in Mutex.
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter); // clone the Arc, not the data
        let handle = thread::spawn(move || {
            // lock() returns a MutexGuard that auto-unlocks on drop.
            // the lock ensures only one thread mutates at a time.
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("counter: {}", *counter.lock().unwrap());

    // channels: typed message passing between threads.
    // mpsc = multiple producer, single consumer.
    let (tx, rx) = mpsc::channel();

    // clone the sender for multiple producers
    let tx2 = tx.clone();

    thread::spawn(move || {
        tx.send("hello from thread 1".to_string()).unwrap();
    });

    thread::spawn(move || {
        tx2.send("hello from thread 2".to_string()).unwrap();
    });

    // rx.recv() blocks until a message arrives.
    // rx is also an iterator — for msg in rx { ... } drains until all senders drop.
    for msg in rx {
        println!("received: {msg}");
    }

    // Send trait: a type can be transferred across thread boundaries.
    // Sync trait: a type can be referenced from multiple threads (&T is Send).
    // most types are Send + Sync. Rc<T> is neither.
    // the compiler checks these automatically — you rarely implement them manually.
}
```

## Modules

Modules organize code into namespaces and control visibility. By default everything is private — you explicitly expose your public API with `pub`.

```rust
// in a single file, mod creates a namespace
mod network {
    // private by default — only accessible within this module
    fn connect_internal() -> bool {
        true
    }

    // pub makes it accessible to parent and sibling modules
    pub fn connect(addr: &str) -> Result<(), String> {
        if connect_internal() {
            Ok(())
        } else {
            Err(format!("failed to connect to {addr}"))
        }
    }

    // pub(crate): visible anywhere in this crate but not to external consumers.
    // useful for internal APIs that multiple modules share.
    pub(crate) fn internal_status() -> &'static str {
        "ok"
    }

    // pub(super): visible to the parent module only
    pub(super) fn debug_info() -> String {
        "network debug".to_string()
    }

    // nested module
    pub mod dns {
        pub fn resolve(host: &str) -> String {
            format!("127.0.0.1 (resolved from {host})")
        }
    }
}

// use brings items into scope
use network::dns;

fn main() {
    network::connect("localhost:8080").unwrap();
    let ip = dns::resolve("example.com");
    println!("{ip}");

    // can also use specific items
    use network::connect;
    connect("0.0.0.0:3000").unwrap();
}

// in a real project, the module tree maps to the filesystem:
//
// src/
//   main.rs          (or lib.rs for libraries)
//   network.rs       (or network/mod.rs)
//   network/
//     dns.rs
//
// main.rs declares: mod network;
// network.rs declares: pub mod dns;
//
// the 2018+ convention prefers network.rs over network/mod.rs
// when the module has submodules, use both:
//   src/network.rs     (declares pub mod dns;)
//   src/network/dns.rs (the dns module)
```

## Testing

Tests live alongside your code. The `#[cfg(test)]` attribute ensures test modules are only compiled when running `cargo test` — zero overhead in production builds.

```rust
// the code under test
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("division by zero".to_string());
    }
    Ok(a / b)
}

pub struct Calculator {
    history: Vec<f64>,
}

impl Calculator {
    pub fn new() -> Self {
        Self { history: Vec::new() }
    }

    pub fn compute(&mut self, value: f64) -> f64 {
        self.history.push(value);
        value
    }

    pub fn last(&self) -> Option<&f64> {
        self.history.last()
    }
}

// test module: only compiled during `cargo test`
#[cfg(test)]
mod tests {
    // bring everything from the parent module into scope
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, 1), 0);
    }

    // assert macros: assert!, assert_eq!, assert_ne!
    // custom messages are optional but help when debugging failures
    #[test]
    fn test_divide() {
        let result = divide(10.0, 3.0).unwrap();
        assert!((result - 3.333).abs() < 0.01, "expected ~3.333, got {result}");
    }

    // testing error cases: assert the function returns Err
    #[test]
    fn test_divide_by_zero() {
        let result = divide(10.0, 0.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "division by zero");
    }

    // #[should_panic]: test that code panics.
    // can optionally check the panic message contains a substring.
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_panic() {
        let v = vec![1, 2, 3];
        let _ = v[99];
    }

    // tests can return Result — lets you use ? in tests
    #[test]
    fn test_with_result() -> Result<(), String> {
        let val = divide(10.0, 2.0)?;
        assert_eq!(val, 5.0);
        Ok(())
    }

    // testing structs with setup
    #[test]
    fn test_calculator() {
        let mut calc = Calculator::new();
        calc.compute(42.0);
        assert_eq!(calc.last(), Some(&42.0));
    }

    // #[ignore]: skip slow tests by default, run with `cargo test -- --ignored`
    #[test]
    #[ignore]
    fn expensive_test() {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

// integration tests live in tests/ directory at the crate root:
//
// tests/
//   integration_test.rs
//
// they can only test your public API (they're external consumers).
// each file in tests/ is compiled as a separate crate.
//
// run specific tests:
//   cargo test test_add              (by name substring)
//   cargo test -- --test-threads=1   (disable parallelism)
//   cargo test -- --nocapture        (show println output)
```
