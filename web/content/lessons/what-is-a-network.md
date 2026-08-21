Have you ever wondered what actually happens when you type `curl https://api.example.com/users` and press Enter? Or why your SSH connection to a remote server just... works? Behind every network request lies a fascinating stack of protocols, each solving a specific piece of a much larger puzzle.

In this lesson, we are going to strip networking back to its fundamentals. No cloud abstractions. No "it just works" hand-waving. We will start with two computers and a wire, then build up our understanding from there.

## Two Computers, One Wire

Imagine you have two computers sitting on a desk, and you want them to communicate. What do you actually need?

At the most basic level, you need:
1. A physical medium to carry signals (a wire, fiber optic cable, or radio waves)
2. A way to encode information onto that medium (electrical voltages, light pulses, or radio frequencies)
3. Agreement on what those signals mean (protocols)

Let us see this in practice. All shell commands in this course target Linux (which is what the lab containers run). If you are following along on macOS, the equivalents are `ifconfig` instead of `ip addr`, `netstat -rn` instead of `ip route`, and `netstat` instead of `ss`. The concepts are identical — only the command names differ.

On a Linux machine, you can examine your network interfaces:

```bash
ip link show
```

Output might look like:

```
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP mode DEFAULT group default qlen 1000
    link/ether 52:54:00:12:34:56 brd ff:ff:ff:ff:ff:ff
```

Two interfaces appear here:
- `lo` - the loopback interface (your computer talking to itself)
- `eth0` - an Ethernet interface (where physical or virtual cables connect)

Notice the `mtu` value on each interface. MTU (Maximum Transmission Unit) is the largest packet an interface can send in a single frame. Ethernet's MTU is 1500 bytes — a physical constraint of the standard. Loopback uses 65536 — a kernel buffer-size choice, not a protocol maximum. The IP total-length field is 16 bits and can represent at most 65,535 bytes, so 65,536 is actually larger than any single IP packet can be. Loopback can exceed this because the kernel never has to put packets on a real wire and can use GSO (Generic Segmentation Offload) to handle oversized buffers internally. If you try to send a packet larger than the MTU, the kernel must either fragment it into smaller pieces or reject it entirely.

The `link/ether 52:54:00:12:34:56` is a MAC address - we will cover these in depth later. For now, just note that every network interface has a unique hardware identifier.

Go exposes the same information through `net.Interfaces()`:

```go
ifaces, err := net.Interfaces()
if err != nil {
    log.Fatal(err)
}
for _, iface := range ifaces {
    addrs, _ := iface.Addrs() // per-interface error unlikely; safe to skip
    fmt.Printf("%s  MAC=%s  MTU=%d  addrs=%v\n",
        iface.Name, iface.HardwareAddr, iface.MTU, addrs)
}
// lo   MAC=         MTU=65536  addrs=[127.0.0.1/8 ::1/128]
// eth0 MAC=52:54:00:12:34:56  MTU=1500  addrs=[172.17.0.2/16]
```

Same data, same interfaces — `ip link show` and `net.Interfaces()` both query the kernel's network stack.

## The Problem of Complexity

Here is the challenge: network communication is genuinely complicated. A single HTTP request involves:
- Converting your application data into bytes
- Ensuring those bytes arrive completely and in order
- Finding a path through potentially dozens of intermediate devices
- Encoding data as electrical signals on copper, light pulses in fiber, or radio waves in the air

If every application had to implement all of this from scratch, we would never get anything done. The solution? Layers.

## The Layered Model

The fundamental insight of network architecture is this: break the problem into layers, where each layer provides services to the layer above it and uses services from the layer below.

Think of it like sending a package internationally:
- You write a letter (your message)
- Put it in an envelope with the recipient's address (addressing)
- The postal service handles routing between countries (routing)
- Trucks, planes, and ships carry the physical package (physical transport)

You do not need to understand how cargo ships navigate to send a letter to Japan. Each layer abstracts away the complexity below it.

The deeper insight is that layers create **contracts between components**. Because each layer only depends on the interface of the layer below — not its implementation — you can swap one for another without disrupting the rest of the stack.

You can see this directly. Run `ip link show` and note whether your active interface is `eth0` (Ethernet) or `wlan0` (Wi-Fi) — these are completely different Layer 2 technologies with different frame formats, different media, and different error correction. Yet `curl https://example.com` returns the same response over either one. The Application layer does not know or care which link type carried its packets. This is also why container networking works at all: Docker and Kubernetes use overlay networks that insert an entirely new layer of encapsulation between IP and Ethernet, and the layers above and below are unaware. Layers allow independent evolution.

### The TCP/IP Model

In practice, we use the TCP/IP model with four layers:

```
+------------------------+
|   Application Layer    |  HTTP, DNS, SSH, SMTP
+------------------------+
|    Transport Layer     |  TCP, UDP
+------------------------+
|    Internet Layer      |  IP, ICMP
+------------------------+
|   Network Access Layer |  Ethernet, Wi-Fi
+------------------------+
```

Let us walk through what each layer does:

**Application Layer** - Where your code lives. When you make an HTTP request, you are working at this layer. You think in terms of URLs, headers, and JSON bodies. You do not think about packets.

**Transport Layer** - Handles end-to-end communication between applications. TCP provides reliable, ordered delivery (your bytes arrive in order, or you know they failed). UDP provides fast, unreliable delivery (great for video streaming where a dropped frame is better than a delayed one).

**Internet Layer** - Handles addressing and routing between networks. This is where IP addresses live. The Internet Layer figures out how to get your packet from your laptop in New York to a server in Tokyo, possibly through dozens of routers.

**Network Access Layer** - Handles communication on a local network segment. Ethernet, Wi-Fi, and the physical transmission of bits all live here.

If you have written Go, you have already used these layers without thinking about them:

```go
// Transport + Internet: bind to a port on all local IPs
ln, err := net.Listen("tcp", ":8080")
if err != nil {
    log.Fatal(err) // e.g. "bind: address already in use" — Layer 4 error
}

// Transport: accept a TCP connection (three-way handshake happens here)
conn, _ := ln.Accept()

// Application: read/write your protocol's data over the connection
buf := make([]byte, 1024)
n, _ := conn.Read(buf)
conn.Write(buf[:n]) // remaining error checks omitted for brevity
```

`net.Listen` asks the kernel to create a socket at the Transport layer and bind it to an IP at the Internet layer. `conn.Read` and `conn.Write` hand bytes to the Transport layer, which handles segmentation, ordering, and retransmission. Everything below — IP routing, Ethernet framing, physical signals — is invisible to your code. That is the layered model at work.

You will hear people reference "Layer 7" or "Layer 4" in conversation — that numbering comes from the OSI model, a more granular 7-layer framework that splits our four layers into seven. The internet settled on TCP/IP because the IETF (the body that standardizes internet protocols) favored "rough consensus and running code" — build it, ship it, iterate. OSI was designed by committee before implementation, and by the time its protocols were ready, TCP/IP already ran the internet. The OSI *model* survived as vocabulary though: Layer 2 means data link (Ethernet/MAC), Layer 3 means network (IP), Layer 4 means transport (TCP/UDP), and Layer 7 means application (HTTP). We will use these numbers throughout the course.

## Why Layers Matter for Debugging

Here is the practical reason to understand layers: when something breaks, you debug layer by layer.

Cannot reach a website? Start from the bottom:
1. **Physical/Network Access**: Is your cable plugged in? Is Wi-Fi connected?
2. **Internet**: Do you have an IP address? Can you ping your gateway?
3. **Transport**: Is something listening on the port? Is a firewall blocking TCP?
4. **Application**: Is the web server returning errors?

This maps directly to error messages you see in Go programs every day:

- `dial tcp 10.0.0.5:5432: connect: connection refused` — **Layer 4**: nothing is listening on that port (is the database running?)
- `dial tcp 10.0.0.5:5432: i/o timeout` — **Layer 3**: route exists but the host is unreachable or a firewall is silently dropping packets
- `dial tcp: lookup myservice.local: no such host` — **DNS (Application layer)**: the hostname cannot be resolved
- `read tcp 127.0.0.1:8080->127.0.0.1:54321: connection reset by peer` — **Layer 4**: the remote side sent a RST (crashed, or rejected the data)
- `no route to host` — **Layer 3**: the kernel has no route to reach that IP at all

Recognizing which layer an error belongs to tells you where to start investigating.

Let us see this in practice. Check your network configuration:

```bash
# view your IP addresses
ip addr show

# check your routing table (how do packets get to their destinations?)
ip route show
```

Now find your default gateway and verify you can reach it:

```bash
# extract the gateway address from the routing table
ip route show default
# output: default via 172.17.0.1 dev eth0
#                      ^^^^^^^^^^
#                      this is your gateway

# ping it (replace with the address from your output)
ping -c 3 172.17.0.1
```

Your gateway address will likely differ — `172.17.0.1` is typical for Docker environments. Once you understand what each step does, you can combine them into a one-liner:

```bash
ping -c 3 "$(ip route get 8.8.8.8 | awk '/via/{print $3; exit}')"
```

`ip route get 8.8.8.8` asks the kernel which route it would actually use for a specific destination — this respects metric, rule, and policy routing, so you always get the preferred gateway. The `awk` filter extracts the `via` field. On a host with multiple default routes, `ip route show default | awk '/default/{print $3}'` picks the first textual match, not the preferred route.

Each command operates at a different layer:
- `ip addr` shows your Network Access and Internet layer configuration
- `ip route` shows Internet layer routing decisions
- `ping` tests Internet layer connectivity using ICMP

## Encapsulation: How Layers Communicate

When data moves down through the layers, each layer adds its own header. This is called **encapsulation**.

Your HTTP request becomes:
```
[Ethernet Header][IP Header][TCP Header][HTTP Data][Ethernet Trailer]
```

When a switch receives this, it only looks at the Ethernet header. When a router receives it, it looks at the IP header. Your web server looks at the TCP and HTTP parts. Each device operates at its appropriate layer.

When data moves up through layers on the receiving end, each layer strips off its header - this is **decapsulation**.

The overhead adds up. An Ethernet header is 14 bytes, IP adds 20, TCP adds 20 — that is 54 bytes of headers before a single byte of your data. Send a 1-byte payload and you have a 55-byte frame: 98% overhead. This is why protocols batch data rather than sending one byte at a time, and why TCP's Nagle algorithm exists (it holds back small writes while previously sent data is unacknowledged, then flushes them together once the ACK arrives). At a full 1460-byte TCP payload inside a 1500-byte MTU frame, overhead drops to ~3.6%.

Encapsulation is what makes the "layers as contracts" idea work in practice. The IP layer does not care whether its payload is TCP, UDP, or something entirely new — it wraps whatever it receives and forwards based on the destination IP. This is why Google could build QUIC (a new transport protocol over UDP) without changing anything at the IP layer, and why your application code does not break when the network switches from Ethernet to Wi-Fi underneath. Each layer treats the layer above as opaque payload.

These layers are not abstract — they are physical bytes on the wire. Here is a simplified view of what a captured packet actually looks like in memory (byte offsets on the left):

```
Byte 0-13:   [dst MAC 6B][src MAC 6B][type 2B]       ← Ethernet header (14 bytes)
Byte 14-33:  [ver/IHL][TOS][len][ID][flags][TTL]      ← IP header (20 bytes)
             [proto][checksum][src IP 4B][dst IP 4B]
Byte 34-53:  [src port 2B][dst port 2B][seq 4B]       ← TCP header (20 bytes minimum; options can extend this)
             [ack 4B][flags][window][checksum][urg]
Byte 54+:    GET /index.html HTTP/1.1\r\n...           ← Application data
```

Each layer's header sits directly in front of the next layer's data — no separators, no metadata, just contiguous bytes. A switch reads bytes 0-13 and stops. A router reads bytes 14-33. Your web server skips to byte 54.

But how does each layer know which protocol comes next? Through **demultiplexing fields** embedded in the headers. The Ethernet header's `type` field (bytes 12-13) contains an EtherType value — `0x0800` means "the payload is IPv4." The IP header's `proto` field tells the kernel what transport protocol follows — `6` means TCP, `17` means UDP. And TCP/UDP port numbers demultiplex at the transport layer — port 80 means HTTP, port 22 means SSH. Each layer uses a single field to dispatch the payload to the correct handler above it.

In a later chapter we will use `tcpdump -X` to inspect real captured packets and verify this structure firsthand.

## Network Devices and Their Layers

Follow a packet as it travels from your laptop to a web server. Each device along the path reads only the headers it understands, acting at its specific layer.

Your packet leaves the laptop's NIC as an Ethernet frame. The first device it hits is a **switch (Layer 2)**. The switch reads the Ethernet header — bytes 0-13 from the diagram above — and looks at the destination MAC address. It maintains a table mapping MAC addresses to physical ports (learned by observing source MACs on incoming frames) and forwards the frame out the correct port. It never looks at the IP header. Before switches, **hubs (Layer 1)** flooded every frame to every port — this is why switches were invented and why broadcast domains matter for ARP.

The frame arrives at a **router (Layer 3)**. The router strips the Ethernet header and reads the IP header — bytes 14-33. It looks up the destination IP in its routing table, decrements the TTL, and re-encapsulates the packet in a new Ethernet frame addressed to the next hop's MAC. The router connects different networks: your LAN to your ISP, your ISP to the backbone.

But how does the router know the next hop's MAC address? This is where **ARP (Address Resolution Protocol)** bridges Layer 2 and Layer 3. When a device knows a destination IP but not the corresponding MAC, it broadcasts an ARP request: "Who has 10.0.0.1?" The device with that IP responds with its MAC address, and the sender caches the result. You can see your machine's ARP cache with:

```bash
ip neigh show
```

Each entry maps an IP address to a MAC address — this is the glue between the Internet layer and the Network Access layer. We will explore ARP in detail in a later chapter.

Other devices operate at higher layers — firewalls can inspect packets from Layer 3 through Layer 7, and load balancers distribute traffic based on transport or application layer information. We will cover these in depth when we reach TCP and HTTP.

## Practical Exercise: See the Layers

Let us see these layers in action with two exercises — one at the Internet Layer (routing), one at the Transport Layer (TCP connections).

### Layer 3: Tracing the Route

Every packet hops through a series of routers to reach its destination. You can see these hops with `traceroute`:

```bash
traceroute -m 5 -n 8.8.8.8
```

How does traceroute discover each router? It exploits the TTL (Time To Live) field in the IP header — the same field we saw in the byte-offset diagram. TTL is a counter that each router decrements by one before forwarding. When TTL hits zero, the router drops the packet and sends back an ICMP "Time Exceeded" message. Traceroute sends its first packet with TTL=1 (so the first router replies), then TTL=2 (second router replies), and so on — building a map of the path hop by hop.

The `-m 5` limits the trace to 5 hops — enough to see the concept without waiting 30 seconds for timeouts. Output shows each router along the path:

```
traceroute to 8.8.8.8 (8.8.8.8), 5 hops max, 60 byte packets
 1  172.17.0.1  0.519 ms  0.505 ms  0.491 ms
 2  10.0.0.1  8.412 ms  8.402 ms  8.391 ms
 ...
```

Each line is a router that your packet passes through. This is the Internet Layer in action — each router makes a forwarding decision based on the destination IP. Inside a lab container, you will typically see only 1-2 real hops before `* * *` entries (routers that do not respond to traceroute probes). That is expected — the point is seeing that intermediate routers exist between you and the destination.

### Layer 4: A TCP Connection

Traceroute showed us Layer 3 routing. Now let us drop down to Layer 4 and watch a TCP connection form. Use both terminals in the lab panel.

Terminal 1 - start a simple server:
```bash
nc -lk 8080
```

The `-l` flag tells netcat to listen, and `-k` keeps it listening after each client disconnects (without `-k`, `nc -l` exits after the first connection closes).

Terminal 2 - connect to it:
```bash
nc localhost 8080
```

Type something in Terminal 2 and watch it appear in Terminal 1. You just communicated across the network stack.

Now press `Ctrl+C` in Terminal 2 to disconnect the client. The server keeps listening (thanks to `-k`). Before reconnecting, check the TCP state in Terminal 2:

```bash
ss -tn | grep 8080
```

The `ss` command shows socket statistics. The `-t` flag filters to TCP sockets, and `-n` shows numeric addresses instead of resolving hostnames. The output columns are: `State` (the TCP state machine), `Recv-Q` (bytes received but not read by the application), `Send-Q` (bytes sent but not acknowledged by the remote side), then `Local Address:Port` and `Peer Address:Port`.

You will likely see:
```
TIME-WAIT 0 0 127.0.0.1:54321 127.0.0.1:8080
```

`TIME-WAIT` is the state a TCP connection enters after closing. The kernel keeps the socket around for ~60 seconds to prevent a subtle bug: if a new connection reuses the same 4-tuple (source IP, source port, destination IP, destination port) immediately, delayed packets from the old connection could be mistaken for the new one. TIME-WAIT ensures old segments expire before the port is reused.

This is why restarting a Go server sometimes fails with `bind: address already in use` — the old socket is still in TIME-WAIT. Go's `net.Listen` sets `SO_REUSEADDR` by default on Unix-like systems, which allows binding to a port in TIME-WAIT, so you rarely hit this in practice. (On Windows, `SO_REUSEADDR` has different — and more permissive — semantics; Go does not set it there.) We will explore TCP connection states in depth in the TCP chapter.

Now connect again and observe `ESTAB` — the state of a live connection. Keep the server running in Terminal 1, then connect from Terminal 2:

```bash
# Terminal 2
nc localhost 8080
```

Type a message to confirm the connection works. Now we need to run `ss` while the connection is still open, but netcat is occupying Terminal 2's foreground. Press `Ctrl+Z` to suspend netcat — this pauses the process but does **not** close the TCP connection (the kernel keeps the socket open):

```bash
# Terminal 2: press Ctrl+Z, then run:
ss -tn | grep 8080
```

```
ESTAB 0 0 127.0.0.1:54322 127.0.0.1:8080
```

`ESTAB` means the TCP three-way handshake completed and data can flow. The `127.0.0.1:8080` is the server (Internet Layer address : Transport Layer port), and `127.0.0.1:54322` is the client's ephemeral port — assigned automatically by the kernel.

Type `fg` to resume netcat, or `Ctrl+C` to close the connection.

### Seeing Encapsulation on the Wire

With netcat still running (server in Terminal 1), use Terminal 2 to capture packets and send data in one command:

```bash
# Terminal 2: capture 3 packets in the background, wait for it to start, then send "hello"
tcpdump -c 3 -X -i lo port 8080 &
sleep 1
echo "hello" | nc localhost 8080
```

The `tcpdump` output shows the raw packet structure — IP header, TCP header, then your "hello" payload as hex bytes. On the loopback interface the link-layer header differs from real Ethernet (loopback uses a simplified pseudo-header rather than the full 14-byte Ethernet frame), but the IP and TCP headers are identical to what you would see on a physical interface. Look for `0x45` near the start — that is the first byte of the IP header, encoding version (4) and IHL (5 × 4 = 20 bytes). The following `00` byte is TOS/DSCP/ECN, not part of the version or length fields.

## Key Takeaways

1. **Networks are layered** - Each layer solves specific problems and provides services to the layer above it.

2. **TCP/IP is practical** - Four layers: Application, Transport, Internet, Network Access. This is what the actual internet uses.

3. **Encapsulation builds packets** - Each layer adds headers. Switches, routers, and servers each look at different headers.

4. **Debug layer by layer** - Start from the bottom (is the cable plugged in?) and work up (is the application returning errors?).

5. **Tools operate at specific layers** - `ip link` (Layer 2), `ip route` (Layer 3), `ss` (Layer 4), `curl` (Layer 7).

## What's Next

Now that you understand the layered model, we will dive into how data actually travels through these layers. In the next lesson, we will look at packets and frames - the fundamental units of network data. You will learn why data is chopped into pieces, what headers contain, and how to tell a frame from a packet from a segment.

## Exercises

1. Run `ip link show` and compare the MTU values of `lo` (65536) and `eth0` (1500). The MTU (Maximum Transmission Unit) is the largest packet size the interface can send in one frame. Try `ping -c 1 -s 2000 -M do localhost` (which forces a 2000-byte payload with "don't fragment"), then try `ping -c 1 -s 2000 -M do <your-gateway-ip>`. Which one succeeds and which one fails? Why?

2. Run `ip route show` and count the number of routes. For each route, identify whether it is a directly connected network (no `via` keyword) or a gateway route (has `via`). What is the difference between these two types?

3. Run `traceroute -m 5 -n 8.8.8.8` and then `traceroute -m 5 -n 1.1.1.1`. Compare the outputs — do they share any hops? What does a shared hop tell you about the path your packets take?

4. Save this Go TCP echo server to a file and run it in Terminal 1:

    ```go
    package main

    import (
        "fmt"
        "io"
        "log"
        "net"
    )

    func main() {
        ln, err := net.Listen("tcp", ":9090")
        if err != nil {
            log.Fatal(err)
        }
        fmt.Println("listening on :9090")
        conn, err := ln.Accept()
        if err != nil {
            log.Fatal(err)
        }
        fmt.Println("connected:", conn.RemoteAddr())
        io.Copy(conn, conn) // echo everything back
        conn.Close()
    }
    ```

    Connect with `nc localhost 9090` from Terminal 2, type a message, and see it echoed back. Check `ss -tn | grep 9090` to verify the `ESTAB` state. What layer does `conn.RemoteAddr()` report from — and what is the ephemeral port?

5. Challenge: Start `nc -lk 9090` in Terminal 1, then in Terminal 2 run `yes | head -c 10M | nc localhost 9090` to blast 10 MB of data. While the transfer is in progress, quickly run `ss -tn | grep 9090` in Terminal 2. Look at the `Recv-Q` and `Send-Q` columns — `Send-Q` shows bytes your kernel has queued for sending but the remote side has not yet acknowledged, and `Recv-Q` shows bytes received by the kernel but not yet read by the application. If you see non-zero values, you are watching TCP flow control in real time: the sender is producing data faster than the receiver can consume it.
