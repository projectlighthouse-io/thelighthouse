Operating system হচ্ছে software আর hardware এর মাঝখানের abstraction layer।

এই sentence টা পড়তে সহজ, বুঝতে কঠিন। প্রত্যেকটা শব্দ — "abstraction", "layer", "software", "hardware" — এতটা weight বহন করে যা শুধু তখনই clear হয় যখন আপনি দেখেন এগুলো ছাড়া কি হয়। তাই OS কি করে সেটা বলার আগে, আমাদের কথা বলতে হবে <term tip="an abstraction hides complexity behind a simpler interface. you interact with the interface without needing to know how the underlying system implements it">abstraction</term> আসলে কি, আর hardware এর কেন একটা দরকার।

## Operating system আসলে কি

মূলত, একটা operating system হচ্ছে এমন একটা program যা আপনার computer এর hardware কে manage করে যাতে অন্য program গুলোকে ওটা না করতে হয়। আপনি যখন machine টা power on করেন, এটাই প্রথম চলে, আর shut down এর সময় এটাই সবার শেষে বন্ধ হয়। আপনি যতবার একটা file open করেন, একটা program run করেন, বা internet এ connect করেন — actual কাজটা OS ই নিচে করে। মানে আপনার request কে আপনার specific hardware যে electrical signals বুঝে সেগুলোতে translate করে দেয়।

একটা translator হিসেবে চিন্তা করুন। আপনার Rust program যখন `std::fs::read_to_string("config.json")` call করে, OS সেই request টা নিয়ে এমন কিছু কাজে break down করে যেগুলো ও delegate করতে পারে। ও filesystem এ filename টা lookup করে, inode খুঁজে বের করে, disk এর কোন blocks এ data আছে সেটা determine করে। But এখানে interesting জিনিস হচ্ছে — OS নিজে actually disk read করে না। ও <term tip="a hardware chip on the storage device that accepts commands (read sector N, write sector M) and manages the physical media — spinning platters on an HDD, NAND cells on an SSD. the CPU does not touch the storage directly; it talks to this controller">disk controller</term> কে বলে blocks গুলো fetch করতে, আর controller টাই physical কাজটা করে। OS শুধু orchestrate করে। ও request টা setup করে, hardware কে বলে data কোথায় RAM এ রাখতে, আর disk controller যখন একটা <term tip="a hardware signal sent by a device to the CPU saying 'I finished what you asked.' the CPU pauses what it is doing, runs a short kernel handler to process the result, and resumes. this is how disk reads, network packets, and keyboard presses get the CPU's attention">interrupt</term> এর মাধ্যমে "done" signal দেয়, OS bytes গুলোকে আপনার program এর buffer এ copy করে। আপনার Rust code একটা `String` ফেরত পায়। ও এসব কিছুই দেখে না। পুরো I/O path — interrupts, DMA, page cache — Chapter 6 এ cover করবো।

## সবসময় OS দরকার হয় না

আমরা এগোনোর আগে, important একটা জিনিস: সবকিছু operating system এ চলে না। আর এটা by design।

Alternative হচ্ছে <term tip="software that runs directly on the hardware without an operating system. the code has direct access to CPU registers, memory addresses, and peripherals. there is no kernel, no scheduler, no virtual memory — just your code and the metal">bare-metal programming</term> — আপনার code মাঝখানে কিছু না রেখে directly hardware এ চলে। কোনো kernel নাই, scheduler নাই, filesystem নাই, virtual memory নাই। আপনি নিজে CPU registers আর peripherals এর সাথে কথা বলেন।

এটা past এর কোনো relic না। আজও পৃথিবীর কিছু critical software এভাবেই চলে।

<div class="border-pencil-solid-amber bg-amber-50/40 rounded-md px-6 py-5 my-6">

**যেসব fields এ operating system প্রায়ই থাকে না:**

- **Embedded systems** — আপনার microwave, গাড়ির ABS controller, thermostat — এগুলোর firmware। এসব microcontroller এ চলে যেগুলোতে 64 KB flash আর 8 KB RAM থাকে। একটা OS এতে fit হবে না।
- **Real-time systems** — flight controllers, industrial robots, medical devices। একটা pacemaker afford করতে পারে না যে heartbeat এর মাঝখানে OS garbage collection চালাবে বা page fault handle করবে। Timing guarantees non-negotiable।
- **Aerospace** — একটা rocket বা satellite এর guidance computer bare-metal বা একটা minimal <term tip="a real-time operating system — a lightweight OS designed for systems where timing is critical. unlike Linux, an RTOS guarantees that a task will complete within a known time bound. examples: FreeRTOS, Zephyr, VxWorks">RTOS</term> এ চলে। 1977 এ launch হওয়া Voyager 1 এখনো 69 KB memory এর একটা processor এ code চালাচ্ছে। ওখানে কোনো Linux নাই।
- **Automotive ECUs** — modern গাড়িতে 50-100 টা <term tip="electronic control unit — a small embedded computer in a vehicle that controls a specific function like engine timing, brake pressure, airbag deployment, or window motors. each ECU runs its own firmware">ECU</term> থাকে। বেশিরভাগ bare-metal firmware বা AUTOSAR চালায়, general-purpose OS না।
- **Bootloaders** — আপনার OS load হওয়ার আগে যে code চলে (GRUB, U-Boot) সেটাও bare-metal। OS start হওয়ার আগে কিছু একটা কে hardware setup করতেই হয়।

</div>

Tradeoff টা straightforward। OS আপনাকে multitasking, memory isolation, device drivers, filesystems দেয় — কিন্তু এর জন্য memory খরচ হয়, latency বাড়ে, আর complexity আসে যেটা হয়তো আপনার দরকার নাই। আপনার পুরো program যখন 4 KB আর 10 microseconds এর মধ্যে respond করতে হবে, তখন OS এমন একটা overhead যেটা আপনি afford করতে পারবেন না।

এই book টা ঐ situation নিয়ে যেখানে আপনার কাছে একটা operating system *আছে* — specifically Linux। কিন্তু এটা যে একটা choice, given না — এই জিনিসটা জানা আপনি OS কে যেভাবে দেখেন সেটাই change করে দেয়। OS machine না। এটা software যা machine এর উপর বসে আছে, আর এটা যা যা provide করে — processes, virtual memory, filesystems — সবই একটা invention, physics এর কোনো law না।

## Abstraction মানে কি

একটা file read করার কথা চিন্তা করুন। আপনার program এ আপনি `std::fs::read_to_string("config.json")` call করেন আর content ফেরত পান। Simple। কিন্তু file টা actually যেভাবে আমরা ভাবি সেভাবে exist করে না। Disk এ কোনো "file" নাই। আছে spinning platter এর উপর magnetic regions, বা NAND flash cells এর charge states — physical sectors এ ছড়ানো যেগুলো হয়তো contiguous ও না। Disk controller একটা protocol এ কথা বলে — SATA, NVMe — যা block addresses আর sector numbers এ deal করে। এর কিছুই "config.json" এর সাথে related না।

আপনার `read_to_string()` call আর ঐ electrical signals এর মাঝে, কিছু একটা translate করে। ও filename টা নিয়ে একটা <term tip="a data structure on disk that stores a file's metadata — size, permissions, timestamps, and pointers to the actual data blocks. the filename is stored separately in a directory entry that points to the inode">inode</term> এ map করে। Inode এর block pointers খুঁজে বের করে। Disk controller কে সঠিক commands issue করে। Data কে memory তে copy করে যেখানে আপনার program read করতে পারে। আপনার program এর কিছুই দেখে না। শুধু bytes পায়।

এটাই abstraction। <span class="highlight-wavy">একটা simpler interface যা একটা more complex reality কে hide করে।</span> আপনি "files" এর সাথে interact করেন। OS interact করে block devices, sector addresses, আর interrupt handlers এর সাথে। এই দুই world এর মাঝের gap টাই operating system fill করে।

## Hardware এর কেন একটা abstraction layer দরকার

একটা CPU instructions execute করে। ব্যাস। এটা memory থেকে next instruction fetch করে, decode করে, execute করে, আর next এ যায়। এটা জানে না "file" কি। জানে না "program" কি। জানে না দুইটা program একে অপরের memory read করা উচিত না। Hardware fast, obedient, আর আমরা যেসব জিনিস granted ধরে নিই সেগুলোর কোনো concept ই এর কাছে নাই।

<div class="border-pencil-solid-amber bg-amber-50/40 rounded-md px-6 py-5 my-6">

একটা abstraction layer ছাড়া, প্রতিটা program কে করতে হতো:

- কোন exact model এর disk controller installed সেটা জানা আর সরাসরি ওটার protocol এ কথা বলা
- Physical memory addresses manage করা, sure করা যাতে দুইটা program একই region use না করে
- CPU share করার জন্য বাকি সব running program এর সাথে coordinate করা
- Keyboard, network card, আর disk এর hardware interrupts handle করা

</div>

এটা একটু দাঁড়িয়ে ভাবার মতো জিনিস। OS ছাড়া, আপনার web server কে rewrite করতে হতো প্রত্যেকবার যখন কেউ একটা SSD swap করে আরেকটা model এর দেয়। দুইটা program একই physical memory addresses এ লিখে একে অপরকে corrupt করে দিতো। একসাথে একাধিক program safely চালানোর কোনো উপায়ই থাকতো না — আর তাদের মধ্যে কোনো isolation ও না। প্রত্যেক programmer ই একজন hardware programmer হতো।

Operating system এই problem টা solve করে আপনার program আর hardware এর মাঝখানে বসে। এটা একটা clean, uniform interface present করে — block addresses এর জায়গায় files, raw instruction streams এর জায়গায় processes, physical RAM এর জায়গায় virtual memory। আপনার program OS এর সাথে কথা বলে। OS hardware এর সাথে কথা বলে। আপনার program কখনো জানতে হয় না কোন hardware এ চলছে।

## The kernel

Operating system এর core হচ্ছে <term tip="the part of the operating system that runs with full hardware privileges. it manages memory, schedules processes, handles interrupts, and provides system calls. everything else — shells, editors, browsers — runs on top of it">kernel</term>। Linux এ (যেটা এই book এ আমরা focus করবো), kernel একটা single program যা machine power on করার সাথে সাথে start হয় আর shut down পর্যন্ত চলে। প্রত্যেকটা hardware resource এর উপর এর direct access আছে: physical memory, CPU registers, disk controllers, network interfaces।

বাকি প্রতিটা program — আপনার shell, editor, web server — একটা restricted mode এ চলে যেখানে hardware কে directly touch করতে পারে না। একটা program যখন একটা file read করতে চায়, একটা process create করতে চায়, বা একটা network packet পাঠাতে চায়, একটা <term tip="a controlled entry point into the kernel. the program places arguments in specific CPU registers and executes a special instruction (syscall on x86-64) that switches the CPU to kernel mode. the kernel validates the request, performs the operation, and returns the result">system call</term> এর মাধ্যমে kernel কে request করে। Kernel request টা validate করে, operation perform করে, আর result return করে। <span class="highlight-wavy">User programs আর kernel এর মাঝের এই boundary টা computing এর সবচেয়ে important boundary</span> — এটাই এক buggy program কে পুরো machine crash করানো থেকে আটকায়।

![userspace to hardware](https://spaces.projectlighthouse.io/books/os-fundamentals/userspace-to-hardware.svg)

এই মুহূর্তে, আপনার machine হয়তো 50 টা program চালাচ্ছে। আপনার browser, editor, একটা terminal, Spotify, background এ একটা database। প্রত্যেকটার নিজস্ব memory address zero থেকে শুরু, নিজস্ব CPU এর slice, নিজস্ব files। কেউ একে অপরের কথা জানে না। আপনার machine এ 8 টা CPU core। 50 টা program। অংকটা মিলে না — তবুও সব কাজ করে। Kernel ই এটা possible করে।

## এই book কি cover করে

এই book টা explore করে আপনার code এর নিচে কি হয়। আপনি যখন একটা file read করতে `open()` call করেন, kernel actually কি করে? আপনি যখন একটা নতুন thread start করেন, CPU ওটা আর অন্য threads এর মধ্যে কিভাবে switch করে? আপনার program যখন memory allocate করে, ওটা কোথা থেকে আসে আর machine যখন run out হয়ে যায় তখন কি হয়?

সাতটা chapter, প্রত্যেকটা আগেরটার উপর build করা:

**Processes & Execution.** কাজের fundamental unit। একটা process কি (এটা "একটা running program" না — সেই definition incomplete)। Kernel internally এটাকে কিভাবে represent করে। `fork()` আর `exec()` কিভাবে নতুন process create করে। Process states, signals, আর inter-process communication। আপনি যদি কখনো ভেবে থাকেন একটা terminal এ command type করলে কি হয়, এই chapter টা instruction by instruction answer দেয়।

**CPU Scheduling.** এক CPU, অনেক process। Kernel কে decide করতে হয় কোন process next চলবে, কতক্ষণ চলবে, আর একটা higher-priority task আসলে কি হবে। আমরা scheduling algorithms cover করবো — simple round-robin থেকে Linux এর CFS আর newer EEVDF scheduler পর্যন্ত। বুঝবেন কেন আপনার program মাঝে মাঝে slow হয় যদিও CPU 100% এ না।

**Memory Management.** প্রত্যেক process ভাবে এর নিজের memory address zero থেকে শুরু। আসলে না — সেটা virtual memory, computing এর সবচেয়ে elegant abstraction গুলোর একটা। Page tables, TLBs, demand paging, swap, OOM killer। এই chapter এর পর, `malloc()` আর কোনো magic না।

**Concurrency & Synchronization.** Multiple threads একই memory share করছে। Race conditions, mutexes, semaphores, deadlocks। কেন concurrent programming hard, hardware help করার জন্য কি provide করে, আর কোন patterns কাজ করে।

**File Systems.** Disk এর উপর bytes কে এমনভাবে organize করা যাতে meaningful কিছু হয়। Inodes, directories, journaling, VFS layer। File টা SSD তে থাকুক, network share এ থাকুক, বা `/proc` এর মতো virtual filesystem এ থাকুক — same `open()`/`read()`/`write()` calls কিভাবে কাজ করে।

**I/O & Storage.** CPU আর outside world এর মধ্যে data কিভাবে move হয়। Interrupts, DMA, block I/O, page cache। কেন একটা file read করা মাঝে মাঝে instant হয় (cache এ ছিল) আর মাঝে মাঝে milliseconds লাগে (disk এ hit করেছে)।

**Virtualization & Containers.** Namespaces আর cgroups — kernel features যা container possible করে। একটা container VM না। এটা একটা regular process যার restricted views আছে। এই chapter এর পর, Docker আর কোনো black box না।

## whoareyou

everyone.

আমার career জুড়ে — professional আর learning years দুটোতেই — OS, networking, আর programming fundamentals যেন engineers দের মধ্যে সবচেয়ে neglected topics মনে হয়েছে। আমাদের মধ্যে যারা C দিয়ে শুরু করেছিলাম, university এ OS, networking — সব cover করা courses নিয়েছিলাম। আমরা exam pass করে এগিয়ে গেছি। বেশিরভাগই আর কখনো ফিরে আসিনি।

<strong>so who are you?</strong> If you write code — বা LLM tools use করে code লেখেন। Backend, infra, robotics, embedded systems, databases, network layers, বা frontend — যেটাই হোক। বা হয়তো কোনো একদিন আপনি Linux এর একজন maintainer হতে চান (আমার মতো)।

Kernel এর সাথে কাজ করতে চান বা না চান, এটা আপনার জন্য একটা profession হতে পারে বা শুধু একটা area of interest। Computer কিভাবে কাজ করে সেটা জানা, ঐ theoretical `1 0 110 1` গুলো একসাথে এসে কিভাবে meaningful কিছুতে পরিণত হয় — এতটা meaningful যে একটা LED জ্বালাতে পারে। সেটা আপনার breadboard এর পাশে থাকুক বা Voyager এর মতো একটা spacecraft এ যা cosmos cross করছে।

আর সবকিছু এই fundamentals এ এসে দাঁড়ায়।

## এই book কিভাবে পড়বেন

শুরু থেকেই শুরু করুন। প্রতিটা chapter আগের chapters assume করে। Processes scheduling এর আগে আসতেই হবে (যা আপনি define করেননি সেটা schedule করবেন কিভাবে)। Memory management concurrency এর আগে আসতেই হবে (threads একটা process এর address space share করে — addresses আগে বুঝতে হবে)। File systems I/O model এর উপর build করা।

কিছু chapter long — deliberately। OS topics neatly পাঁচ-minute reads এ decompose হয় না। Virtual memory এর কথাই ধরুন — এটাকে whole হিসেবে বোঝার জন্য page tables, TLBs, demand paging, আর OOM killer দরকার। ছোট ছোট piece এ split করলে flow break হয়ে যাবে আর প্রত্যেকবার ফিরে এসে context আবার build করতে হবে। তাই আমরা chapter গুলোকে complete units of understanding হিসেবে রাখি। যদি একবারে পড়ে শেষ করতে না পারেন, <span class="highlight-wavy">bookmark feature use করুন</span> — এটা আপনার reading position save করে রাখে যাতে exactly যেখানে ছেড়েছিলেন সেখান থেকে শুরু করতে পারেন।

Chapter গুলো যথেষ্ট independent যাতে কোনো topic আপনাকে call করলে আপনি আগে যেতে পারেন, কিন্তু মাঝে মাঝে আগে cover হওয়া concepts এর forward references এ hit করবেন। Book টা order এ পড়ার জন্য designed।

পড়ার সময় একটা terminal open রাখুন। Commands run করুন। `ps`, `top`, `strace`, `pmap`, `lsof` use করুন। Kernel abstract না — এটা এই মুহূর্তে আপনার machine এ চলছে, আপনার processes manage করছে, memory page করছে, threads schedule করছে। এই book এর প্রতিটা concept ই সঠিক tool দিয়ে observe করা যায়।
