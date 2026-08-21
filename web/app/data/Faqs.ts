import type { Faq } from '@/types/Content'

// Shared: the section renders these and the page emits them as FAQPage
// structured data. Two copies would drift and Google would flag the schema
// as not matching the visible page.

export const faqs: Faq[] = [
  {
    question: 'Why is it article-based instead of video?',
    answer: `When you read, your brain needs to take the cognitive load. When you're watching videos, it's often spoon feeding. Ever thought about why our attention span is in 2-digit seconds nowadays? The aim is to be a builder, not a watcher.

Articles let you learn at your own pace. You can skim, re-read, and jump around easily. You can highlight text, take notes in the margins, and search for keywords. You can also read offline or in low-bandwidth situations.

Also, one of the biggest myths is that videos are better for learning. Research shows that active reading leads to better retention and understanding than passive watching.`,
  },
  {
    question: 'Why did you build this platform?',
    answer: `There's a big gap between using Docker and understanding how containers actually work. Between running a service and understanding the networking stack beneath it. Between writing code and knowing what the OS does with it.

I'm building this platform to help bridge that gap. It won't solve everything, but it might help developers build a foundation for understanding the infrastructure they work with every day.

Because without some of that foundation, it's easy to end up blindly accepting whatever comes your way—even 'sudo rm -rf ~/'.`,
  },
  {
    question: 'Who is this for?',
    answer: `projectlighthouse is for the seekers. The ones who want to understand how things work under the hood. The ones who want to go beyond just using tools and platforms, and actually understand the infrastructure they interact with daily.

You might be actively learning by building things, reading documentation, or challenging yourself with hands-on labs. Either way, if you have that curiosity and drive to understand how systems actually work, this is for you.

If you've ever wondered how Docker actually isolates processes, why your TCP connection hangs, how memory allocation works, or what the OS does when you open a file—this is for you.`,
  },
  {
    question: 'Why only Go, Rust, and probably C?',
    answer: `Language doesn't matter—except when it does. For systems and infrastructure work, it absolutely matters.

Go powers Docker, Terraform, containerd, and half the cloud-native tools you use daily. Its simplicity and concurrency model make it the lingua franca of cloud infrastructure.

Rust offers memory safety without a garbage collector, making it ideal for performance-critical infrastructure components. It's being adopted widely in the industry—from the Linux kernel to new container runtimes and networking tools.

C provides low-level access to understand how containers interact with the kernel, how namespaces work, and what's really happening under the hood.

As projectlighthouse is still a one-man show, I had to pick languages that are industry-standard and that I work with daily—I can't teach something I don't know.`,
  },
  {
    question: "I can't afford a subscription right now. Is there another way?",
    answer: `Yes. We have the Lady Elizabeth scholarship. If you're going through a tough time and believe the content here can help you grow, reach out. Share what you've done and how these books, labs, or projects would benefit you. We'll grant access to relevant material or waive part or full payment. Don't let cost be a blocker.`,
  },
  {
    question: 'Do I need Linux?',
    answer: `For the articles, no—you can read and follow along on any OS.

For the projects, you'll run them on your own machine with luxctl, our open-source CLI. Most of them are happy on macOS or Linux. The container and kernel-level ones genuinely need Linux—a VM, WSL2 on Windows, or a Linux machine will serve you well.

Containers are inherently Linux-centric. Namespaces, cgroups, the /proc filesystem, container runtimes—these are Linux concepts. You can't learn them properly without touching Linux.`,
  },
]
