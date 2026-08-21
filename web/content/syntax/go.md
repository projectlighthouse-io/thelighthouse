## Hello World

Every Go file belongs to a package. Executables use `package main` with a `func main` entry point.

```go
package main

// parenthesized imports are idiomatic — one import block, not multiple import statements
import (
	"fmt"
	"os"
)

func main() {
	fmt.Println("hello, world")

	// os.Args[0] is the binary name, real args start at index 1
	if len(os.Args) > 1 {
		fmt.Println("first arg:", os.Args[1])
	}
}
```

## Variables

Go has two declaration styles. Use `:=` inside functions for brevity, `var` at package level or when you need an explicit type.

```go
package main

import "fmt"

// package-level variables must use var — := is only valid inside functions
var version = "1.0.0"

// const values must be known at compile time — no function calls allowed
const (
	maxRetries = 3
	timeout    = 30 // seconds
)

// iota auto-increments within a const block, starting at 0
// this is the standard pattern for enumerated constants in Go
const (
	StatusPending  = iota // 0
	StatusRunning         // 1
	StatusComplete        // 2
	StatusFailed          // 3
)

func main() {
	// := declares and assigns in one step — type is inferred from the right side
	name := "lighthouse"
	count := 42
	ratio := 3.14 // inferred as float64, not float32

	// var with explicit type — useful when zero value matters or type isn't obvious
	var elapsed float64 // zero value: 0.0
	var ready bool      // zero value: false
	var label string    // zero value: "" (empty string)
	var ptr *int        // zero value: nil

	// multiple assignment — common for swaps and function returns
	x, y := 10, 20
	x, y = y, x

	// blank identifier _ discards a value — compiler enforces that all variables are used
	_, err := fmt.Println(name, count, ratio, elapsed, ready, label, ptr, x, y)
	if err != nil {
		panic(err)
	}
}
```

## Types

Go is statically typed. There are no implicit conversions between numeric types — you must be explicit.

```go
package main

import (
	"fmt"
	"math"
)

func main() {
	// integer types: int (platform-dependent, 64-bit on modern systems), int8, int16, int32, int64
	// unsigned: uint, uint8, uint16, uint32, uint64
	var age int = 30
	var port uint16 = 8080

	// float64 is the default for literals like 3.14 — use float32 only when memory matters
	var pi float64 = math.Pi

	// bool — only true or false, no truthy/falsy like JS or Python
	var active bool = true

	// byte is an alias for uint8 — used for raw data
	// rune is an alias for int32 — represents a Unicode code point
	var b byte = 'A'    // 65
	var r rune = '日'    // 26085

	// type conversions must be explicit — Go won't silently widen or narrow
	var i int = 42
	var f float64 = float64(i) // int → float64
	var u uint = uint(f)       // float64 → uint (truncates decimal)

	// this would NOT compile: var f2 float64 = i
	// Go refuses to lose precision silently

	// string ↔ []byte conversions copy data — they're different memory layouts
	s := "hello"
	bs := []byte(s)  // string → byte slice (copies)
	s2 := string(bs) // byte slice → string (copies)

	fmt.Println(age, port, pi, active, b, r, f, u, s2)
}
```

## Strings

Strings in Go are immutable byte slices. A `string` is UTF-8 encoded, but indexing gives you bytes, not characters. Use `range` or the `unicode/utf8` package for proper Unicode handling.

```go
package main

import (
	"fmt"
	"strings"
)

func main() {
	// double quotes for interpreted strings (escape sequences work)
	greeting := "hello\nworld"

	// backticks for raw strings — no escapes, can span multiple lines
	// great for regex, SQL, JSON templates
	query := `SELECT *
FROM users
WHERE active = true`

	// len() returns byte count, not character count
	emoji := "Go is 🔥"
	fmt.Println(len(emoji)) // 10 (🔥 is 4 bytes in UTF-8), not 6

	// range over a string iterates by rune (Unicode code point), not by byte
	for i, ch := range "café" {
		// i is the byte offset, ch is the rune value
		fmt.Printf("byte %d: %c (U+%04X)\n", i, ch, ch)
		// byte 0: c, byte 1: a, byte 2: f, byte 3: é (2 bytes, next index will be 5)
	}

	// strings package covers most common operations
	fmt.Println(strings.Contains("seafood", "foo"))    // true
	fmt.Println(strings.HasPrefix("/api/v1", "/api"))  // true
	fmt.Println(strings.Split("a,b,c", ","))           // [a b c]
	fmt.Println(strings.Join([]string{"a", "b"}, "-")) // a-b
	fmt.Println(strings.TrimSpace("  hello  "))        // "hello"
	fmt.Println(strings.ReplaceAll("aaa", "a", "b"))   // "bbb"

	// fmt.Sprintf for formatted strings — like printf but returns a string
	name := "lighthouse"
	url := fmt.Sprintf("https://%s.io/api/v%d", name, 1)
	fmt.Println(url) // https://lighthouse.io/api/v1

	_ = greeting
	_ = query
}
```

## Arrays and Slices

Arrays have fixed size and are rarely used directly. Slices are the workhorse — dynamic, reference-backed views into arrays.

```go
package main

import "fmt"

func main() {
	// arrays — fixed size, part of the type signature
	// [3]int and [4]int are completely different types
	var grid [3]int             // [0, 0, 0]
	rgb := [3]byte{255, 128, 0}

	// slices — dynamic, backed by an underlying array
	// this is what you'll use 99% of the time
	nums := []int{1, 2, 3, 4, 5}

	// make([]T, length, capacity) — preallocate when you know the size
	// avoids repeated allocations during append
	buf := make([]byte, 0, 1024)

	// append returns a new slice — the original may or may not be the same backing array
	// always reassign: s = append(s, ...) — never ignore the return value
	nums = append(nums, 6, 7)
	nums = append(nums, []int{8, 9}...) // append another slice with ...

	// slice expressions: s[low:high] — low inclusive, high exclusive
	first3 := nums[:3]  // [1, 2, 3]
	last3 := nums[len(nums)-3:] // [7, 8, 9]

	// WARNING: slices share underlying memory — modifying one can affect the other
	a := []int{1, 2, 3, 4}
	b := a[:2]  // b = [1, 2], shares memory with a
	b[0] = 99   // a is now [99, 2, 3, 4] — this is a common source of bugs

	// copy() for a true independent copy
	src := []int{1, 2, 3}
	dst := make([]int, len(src))
	copy(dst, src)          // dst is independent of src
	dst[0] = 99             // src is still [1, 2, 3]

	// nil slice vs empty slice — both have len 0 but behave slightly differently
	var nilSlice []int          // nil, len=0, cap=0
	emptySlice := []int{}       // not nil, len=0, cap=0
	fmt.Println(nilSlice == nil) // true
	fmt.Println(emptySlice == nil) // false
	// both work fine with append, range, and len — prefer nil slice as zero value

	// delete element at index i (order-preserving) — no built-in for this
	s := []string{"a", "b", "c", "d"}
	i := 2
	s = append(s[:i], s[i+1:]...) // removes "c" → [a, b, d]

	fmt.Println(grid, rgb, buf, first3, last3, a, src, dst, s)
}
```

## Maps

Maps are hash tables. They're reference types — passing a map to a function gives it access to the same data.

```go
package main

import "fmt"

func main() {
	// literal initialization
	status := map[string]int{
		"ok":      200,
		"created": 201,
		"not_found": 404,
	}

	// make for an empty map with optional size hint
	users := make(map[int]string, 100)
	users[1] = "alice"
	users[2] = "bob"

	// access — returns zero value if key is missing, which can be ambiguous
	fmt.Println(status["ok"])      // 200
	fmt.Println(status["missing"]) // 0 — but is it 0 or missing?

	// comma-ok idiom — the only reliable way to check existence
	val, exists := status["missing"]
	if !exists {
		fmt.Println("key not found, val is zero value:", val)
	}

	// common pattern: check and act in one statement
	if port, ok := status["ok"]; ok {
		fmt.Println("found:", port)
	}

	// delete — safe to call on missing keys, won't panic
	delete(status, "not_found")

	// iteration order is intentionally randomized by the runtime
	// never depend on map iteration order
	for code, text := range map[int]string{200: "OK", 404: "Not Found"} {
		fmt.Printf("%d: %s\n", code, text)
	}

	// maps are not safe for concurrent read/write — use sync.Map or a mutex
	// the runtime will panic if it detects concurrent map access
}
```

## Control Flow

Go has no while, no do-while, no ternary operator. `for` is the only loop keyword, but it covers all cases.

```go
package main

import (
	"fmt"
	"runtime"
)

func main() {
	// if — parentheses are not used, braces are required
	x := 10
	if x > 5 {
		fmt.Println("big")
	} else if x > 0 {
		fmt.Println("small")
	} else {
		fmt.Println("zero or negative")
	}

	// if with init statement — variable is scoped to the if/else chain
	// this is idiomatic for error checks and map lookups
	if err := doSomething(); err != nil {
		fmt.Println("error:", err)
	}
	// err is not accessible here — keeps the scope tight

	// classic for loop
	for i := 0; i < 5; i++ {
		fmt.Println(i)
	}

	// while-style: for with only a condition
	n := 1
	for n < 100 {
		n *= 2
	}

	// infinite loop — use break or return to exit
	// common in servers, event loops, retry logic
	attempts := 0
	for {
		attempts++
		if attempts >= 3 {
			break
		}
	}

	// range — works on slices, arrays, maps, strings, channels
	ports := []int{8080, 8081, 8082}
	for i, port := range ports {
		fmt.Printf("index %d: port %d\n", i, port)
	}
	// use _ to discard index: for _, port := range ports { ... }

	// switch — no fallthrough by default (unlike C), no break needed
	switch runtime.GOOS {
	case "linux":
		fmt.Println("linux")
	case "darwin":
		fmt.Println("macOS")
	default:
		fmt.Println("other")
	}

	// switch with no condition — cleaner than long if/else chains
	score := 85
	switch {
	case score >= 90:
		fmt.Println("A")
	case score >= 80:
		fmt.Println("B")
	default:
		fmt.Println("C")
	}

	// type switch — used with interfaces to branch on concrete type
	var val interface{} = "hello"
	switch v := val.(type) {
	case string:
		fmt.Println("string:", v)
	case int:
		fmt.Println("int:", v)
	default:
		fmt.Printf("unexpected type: %T\n", v)
	}
}

func doSomething() error {
	return nil
}
```

## Functions

Functions are first-class values. Multiple return values replace exceptions and out-parameters.

```go
package main

import (
	"errors"
	"fmt"
)

// multiple return values — the (value, error) pattern is everywhere in Go
func divide(a, b float64) (float64, error) {
	if b == 0 {
		return 0, errors.New("division by zero")
	}
	return a / b, nil
}

// named return values — useful for documenting what's returned
// the naked return (just "return") uses the named values — avoid in long functions
func parseCoord(s string) (lat, lon float64, err error) {
	// named returns are pre-initialized to zero values
	_, err = fmt.Sscanf(s, "%f,%f", &lat, &lon)
	return // returns lat, lon, err implicitly
}

// variadic functions — last parameter gets ... prefix, received as a slice
func sum(nums ...int) int {
	total := 0
	for _, n := range nums {
		total += n
	}
	return total
}

// functions are values — can be assigned, passed, returned
func makeMultiplier(factor int) func(int) int {
	// the returned closure captures 'factor' from the enclosing scope
	return func(x int) int {
		return x * factor
	}
}

func main() {
	result, err := divide(10, 3)
	if err != nil {
		fmt.Println("error:", err)
		return
	}
	fmt.Println(result)

	// variadic call — pass individual args or expand a slice with ...
	fmt.Println(sum(1, 2, 3))       // 6
	nums := []int{4, 5, 6}
	fmt.Println(sum(nums...))        // 15

	// closure
	double := makeMultiplier(2)
	fmt.Println(double(5)) // 10

	// defer — runs when the surrounding function returns, not the block
	// defers execute in LIFO order (last deferred = first executed)
	// common for cleanup: closing files, releasing locks, recovering from panics
	fmt.Println("start")
	defer fmt.Println("deferred: runs last")
	fmt.Println("end")
	// output: start, end, deferred: runs last

	// defer with a closure to capture final value of a variable
	x := 0
	defer func() {
		fmt.Println("deferred x:", x) // prints 5, not 0 — closure captures by reference
	}()
	x = 5
}
```

## Pointers

Go has pointers but no pointer arithmetic. They're used to share data and avoid copying large structs.

```go
package main

import "fmt"

// pointer parameter — lets the function modify the caller's data
// without a pointer, Go passes a copy and changes are lost
func increment(val *int) {
	*val++ // dereference and modify
}

// returning a pointer to a local variable is safe in Go
// the compiler detects this and allocates on the heap (escape analysis)
func newConfig(port int) *Config {
	cfg := Config{Port: port, Host: "localhost"}
	return &cfg // safe — Go's garbage collector manages the memory
}

type Config struct {
	Host string
	Port int
}

func main() {
	x := 10
	increment(&x) // pass address of x
	fmt.Println(x) // 11

	// & takes address, * dereferences
	ptr := &x
	fmt.Println(*ptr) // 11
	*ptr = 20
	fmt.Println(x) // 20

	// new() allocates zeroed memory and returns a pointer
	// rarely used — composite literals with & are more common
	p := new(int)   // *int, points to 0
	fmt.Println(*p) // 0

	// idiomatic: &Type{} instead of new(Type)
	cfg := &Config{Port: 8080, Host: "0.0.0.0"}
	fmt.Println(cfg.Port) // Go auto-dereferences struct pointers — no -> operator

	// nil pointers panic on dereference — always check if a pointer could be nil
	var np *Config
	if np != nil {
		fmt.Println(np.Port)
	}
}
```

## Structs

Structs are the building block for custom types. Go uses composition (embedding) instead of inheritance.

```go
package main

import (
	"encoding/json"
	"fmt"
	"time"
)

// struct tags control serialization — json, db, yaml, validate are common
type User struct {
	ID        int       `json:"id"`
	Email     string    `json:"email"`
	Name      string    `json:"name,omitempty"` // omitted from JSON if empty
	CreatedAt time.Time `json:"created_at"`
	password  string    // lowercase = unexported, invisible outside this package
}

// methods are functions with a receiver argument
// value receiver — operates on a copy, can't modify the original
func (u User) DisplayName() string {
	if u.Name != "" {
		return u.Name
	}
	return u.Email
}

// pointer receiver — can modify the struct, avoids copying
// rule of thumb: if any method needs a pointer receiver, make them all pointer receivers
func (u *User) SetEmail(email string) {
	u.Email = email
}

// embedding — composition over inheritance
// Server "has a" Logger, and Logger's methods are promoted to Server
type Logger struct{}
func (l Logger) Log(msg string) { fmt.Println("[LOG]", msg) }

type Server struct {
	Logger           // embedded — no field name, methods are promoted
	Port   int
	debug  bool
}

func main() {
	// struct literal — fields not specified get zero values
	u := User{
		ID:    1,
		Email: "alice@example.com",
		// Name is "" (zero value), CreatedAt is zero time
	}
	fmt.Println(u.DisplayName()) // alice@example.com

	u.SetEmail("bob@example.com")

	// JSON marshal uses struct tags
	data, _ := json.Marshal(u)
	fmt.Println(string(data))

	// embedding in action — call Logger.Log directly on Server
	srv := Server{Port: 8080}
	srv.Log("server starting") // promoted from embedded Logger

	// anonymous structs — useful for one-off JSON decoding, test data
	resp := struct {
		Status string `json:"status"`
		Code   int    `json:"code"`
	}{
		Status: "ok",
		Code:   200,
	}
	fmt.Println(resp.Status)
}
```

## Interfaces

Interfaces are satisfied implicitly — no `implements` keyword. If a type has the right methods, it implements the interface. This is Go's version of polymorphism.

```go
package main

import (
	"fmt"
	"io"
	"strings"
)

// small interfaces are idiomatic — one or two methods
// the stdlib is full of these: io.Reader, io.Writer, fmt.Stringer, error
type Storage interface {
	Get(key string) ([]byte, error)
	Put(key string, value []byte) error
}

// MemoryStorage satisfies Storage without declaring it anywhere
type MemoryStorage struct {
	data map[string][]byte
}

func NewMemoryStorage() *MemoryStorage {
	return &MemoryStorage{data: make(map[string][]byte)}
}

func (m *MemoryStorage) Get(key string) ([]byte, error) {
	val, ok := m.data[key]
	if !ok {
		return nil, fmt.Errorf("key %q not found", key)
	}
	return val, nil
}

func (m *MemoryStorage) Put(key string, value []byte) error {
	m.data[key] = value
	return nil
}

// accept interfaces, return concrete types — this is the Go design guideline
// callers can pass any Storage implementation
func cacheUser(store Storage, id string, data []byte) error {
	return store.Put("user:"+id, data)
}

// the empty interface (any) accepts any type — use sparingly
// 'any' is an alias for interface{} (added in Go 1.18)
func printType(v any) {
	fmt.Printf("type: %T, value: %v\n", v, v)
}

// type assertion — extract the concrete type from an interface
func readAll(r io.Reader) string {
	// r might be a *strings.Reader, *os.File, *bytes.Buffer, etc.
	buf := new(strings.Builder)
	io.Copy(buf, r)
	return buf.String()
}

func main() {
	store := NewMemoryStorage()
	cacheUser(store, "42", []byte(`{"name":"alice"}`))

	val, _ := store.Get("user:42")
	fmt.Println(string(val))

	printType(42)
	printType("hello")

	// type assertion with comma-ok — safe, won't panic
	var i interface{} = "hello"
	s, ok := i.(string)
	if ok {
		fmt.Println("string value:", s)
	}

	// without comma-ok, a failed assertion panics
	// n := i.(int) // PANIC: interface conversion: interface is string, not int
}
```

## Error Handling

Go uses explicit error returns instead of exceptions. The `error` interface is just `Error() string`. Wrapping errors with `%w` creates chains you can inspect with `errors.Is` and `errors.As`.

```go
package main

import (
	"errors"
	"fmt"
	"os"
)

// sentinel errors — package-level vars for known error conditions
// callers check with errors.Is
var (
	ErrNotFound     = errors.New("not found")
	ErrUnauthorized = errors.New("unauthorized")
)

// custom error type — implements the error interface
// use when you need to carry structured data with the error
type ValidationError struct {
	Field   string
	Message string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("validation failed on %s: %s", e.Field, e.Message)
}

func findUser(id int) (string, error) {
	if id <= 0 {
		// %w wraps the error — preserves the chain for errors.Is/As
		return "", fmt.Errorf("findUser(%d): %w", id, ErrNotFound)
	}
	return "alice", nil
}

func validateAge(age int) error {
	if age < 0 || age > 150 {
		return &ValidationError{Field: "age", Message: "must be between 0 and 150"}
	}
	return nil
}

func main() {
	// standard error check — you'll write this hundreds of times
	name, err := findUser(-1)
	if err != nil {
		// errors.Is checks anywhere in the error chain (unwraps %w wrappers)
		if errors.Is(err, ErrNotFound) {
			fmt.Println("user not found")
		} else {
			fmt.Println("unexpected error:", err)
		}
	}
	_ = name

	// errors.As extracts a specific error type from the chain
	if err := validateAge(-5); err != nil {
		var ve *ValidationError
		if errors.As(err, &ve) {
			fmt.Printf("field: %s, message: %s\n", ve.Field, ve.Message)
		}
	}

	// wrapping adds context at each layer — builds a readable error chain
	// "open config: read file: open /etc/app.conf: no such file or directory"
	_, err = os.Open("/etc/nonexistent.conf")
	if err != nil {
		wrapped := fmt.Errorf("open config: read file: %w", err)
		fmt.Println(wrapped)
		fmt.Println(errors.Is(wrapped, os.ErrNotExist)) // true — unwraps through the chain
	}
}
```

## Goroutines and Channels

Goroutines are lightweight threads managed by the Go runtime (not OS threads). Channels are typed pipes for safe communication between goroutines.

```go
package main

import (
	"fmt"
	"sync"
	"time"
)

// worker pattern — goroutines pull from a shared channel
func worker(id int, jobs <-chan int, results chan<- int) {
	for job := range jobs {
		// simulate work
		time.Sleep(100 * time.Millisecond)
		results <- job * 2
	}
}

func main() {
	// launching a goroutine — just prefix the call with 'go'
	// the runtime multiplexes thousands of goroutines onto a few OS threads
	go func() {
		fmt.Println("running in background")
	}()

	// channels — typed conduits for goroutine communication
	// unbuffered channel: sender blocks until receiver is ready (and vice versa)
	ch := make(chan string)
	go func() {
		ch <- "hello from goroutine"
	}()
	msg := <-ch // blocks until a value is sent
	fmt.Println(msg)

	// buffered channel — sender doesn't block until buffer is full
	// useful when producer and consumer run at different speeds
	tasks := make(chan int, 10)

	// fan-out: multiple workers reading from the same channel
	results := make(chan int, 10)
	for w := 0; w < 3; w++ {
		go worker(w, tasks, results)
	}

	// send work
	for j := 0; j < 5; j++ {
		tasks <- j
	}
	close(tasks) // closing signals workers to stop (range will exit)

	// collect results
	for i := 0; i < 5; i++ {
		fmt.Println(<-results)
	}

	// select — like switch but for channel operations
	// blocks until one case is ready; if multiple are ready, picks randomly
	tick := time.Tick(100 * time.Millisecond)
	timeout := time.After(350 * time.Millisecond)
loop:
	for {
		select {
		case <-tick:
			fmt.Println("tick")
		case <-timeout:
			fmt.Println("timed out")
			break loop // label needed — bare break only exits select, not for
		}
	}

	// sync.WaitGroup — wait for a group of goroutines to finish
	var wg sync.WaitGroup
	for i := 0; i < 5; i++ {
		wg.Add(1) // increment before launching the goroutine, not inside it
		go func(id int) {
			defer wg.Done()
			fmt.Printf("worker %d done\n", id)
		}(i) // pass i as argument — capturing the loop variable directly is a common bug
	}
	wg.Wait() // blocks until counter reaches 0

	// sync.Mutex — protect shared state when channels aren't the right fit
	var mu sync.Mutex
	counter := 0
	var wg2 sync.WaitGroup
	for i := 0; i < 1000; i++ {
		wg2.Add(1)
		go func() {
			defer wg2.Done()
			mu.Lock()
			counter++ // without the mutex, this would be a data race
			mu.Unlock()
		}()
	}
	wg2.Wait()
	fmt.Println("counter:", counter) // always 1000
}
```

## Packages and Imports

Go organizes code into packages. Capitalized names are exported (public), lowercase are unexported (private). There are no access modifiers — just naming.

```go
package main

import (
	// standard library
	"fmt"
	"net/http"

	// blank import — only runs the package's init() functions
	// common for database drivers that register themselves
	_ "github.com/lib/pq"

	// aliased import — resolve name collisions or shorten long paths
	rand "crypto/rand"
	mrand "math/rand"
)

// init() runs automatically before main(), after all package-level variables are initialized
// each file can have its own init(), they run in source file order
// use sparingly — implicit behavior is harder to reason about
func init() {
	fmt.Println("init runs before main")
}

func main() {
	// exported names start with uppercase — this is enforced by the compiler
	// http.ListenAndServe is exported, http.defaultServeMux is not
	_ = http.StatusOK     // accessible — exported
	// _ = http.statusText // would not compile — unexported

	// using aliased imports
	_ = rand.Reader       // crypto/rand
	_ = mrand.Intn(100)   // math/rand

	// import path = module path + relative directory
	// example: "github.com/yourname/project/internal/auth"
	// the package name (what you use in code) is the last segment by convention
	// auth.Validate(...) — not internal.auth.Validate
}

// package layout conventions:
//
// project/
// ├── cmd/server/main.go    ← entry point, package main
// ├── internal/auth/         ← private to this module — cannot be imported externally
// ├── internal/db/
// ├── pkg/validator/         ← reusable library code (optional convention)
// ├── go.mod                 ← module declaration + dependencies
// └── go.sum                 ← dependency checksums (committed to VCS)
//
// internal/ is enforced by the Go toolchain — external modules cannot import it
```

## Testing

Go has testing built into the toolchain. No framework needed — just `testing` package and `go test`.

```go
package math

import (
	"testing"
)

// function under test
func Add(a, b int) int {
	return a + b
}

func Abs(n int) int {
	if n < 0 {
		return -n
	}
	return n
}

// test functions must start with Test and take *testing.T
// file must be named *_test.go — these files are excluded from production builds
func TestAdd(t *testing.T) {
	got := Add(2, 3)
	want := 5
	if got != want {
		// t.Errorf continues execution, t.Fatalf stops the test immediately
		t.Errorf("Add(2, 3) = %d, want %d", got, want)
	}
}

// table-driven tests — the dominant pattern in Go
// each case is explicit, easy to add new ones, clear failure messages
func TestAbs(t *testing.T) {
	tests := []struct {
		name string
		input int
		want  int
	}{
		{"positive", 5, 5},
		{"negative", -3, 3},
		{"zero", 0, 0},
	}

	for _, tt := range tests {
		// t.Run creates a subtest — shows up as TestAbs/positive, TestAbs/negative, etc.
		// run a specific one with: go test -run TestAbs/negative
		t.Run(tt.name, func(t *testing.T) {
			got := Abs(tt.input)
			if got != tt.want {
				t.Errorf("Abs(%d) = %d, want %d", tt.input, got, tt.want)
			}
		})
	}
}

// benchmarks — function name starts with Benchmark, takes *testing.B
// run with: go test -bench=. -benchmem
func BenchmarkAdd(b *testing.B) {
	// b.N is adjusted by the framework to get stable timing
	for i := 0; i < b.N; i++ {
		Add(42, 58)
	}
}

// TestMain — controls test setup/teardown for the entire package
// useful for database connections, temp directories, etc.
// func TestMain(m *testing.M) {
//     // setup
//     db := setupTestDB()
//     code := m.Run() // runs all tests
//     db.Close()      // teardown
//     os.Exit(code)
// }

// run tests:
//   go test ./...              — all packages
//   go test -v ./pkg/math/     — verbose, specific package
//   go test -run TestAbs       — filter by name (regex)
//   go test -race ./...        — enable race detector (catches concurrency bugs)
//   go test -cover ./...       — show coverage percentage
//   go test -count=1 ./...     — disable test caching
```
