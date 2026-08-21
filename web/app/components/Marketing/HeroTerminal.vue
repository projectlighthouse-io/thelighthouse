<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';

type Token = { text: string; cls?: string };

type Line =
    | { type: 'line'; tokens: Token[]; delay?: number; typed?: boolean }
    | { type: 'summary'; result: 'pass' | 'fail'; pass: number; slow: number; fail: number; total: number; delay?: number }
    | { type: 'verdict'; result: 'pass' | 'fail'; grade?: string; xp?: number; rank?: number; project: string; delay?: number }
    | { type: 'tip'; lead: string; cmd: string; tail: string; delay?: number };

interface Scenario {
    titlebar: string;
    lines: Line[];
}

const C = {
    dim: 'text-term-dim',
    text: 'text-term-text',
    green: 'text-term-green',
    red: 'text-term-red',
    yellow: 'text-term-yellow',
    cyan: 'text-term-blue',
    orange: 'text-term-orange',
    pink: 'text-term-pink',
    light: 'text-term-light',
};

const promptLine = (user: string, host: string, path: string): Line => ({
    type: 'line',
    tokens: [
        { text: `${user}@${host}`, cls: C.dim },
        { text: ' in ', cls: C.dim },
        { text: path, cls: C.text },
    ],
    delay: 200,
});

const cmd = (text: string, delay = 400): Line => ({
    type: 'line',
    tokens: [
        { text: '$ ', cls: C.dim },
        { text, cls: C.light },
    ],
    delay,
    typed: true,
});

const arrow = (text: string, highlights: Token[] = [], delay = 350): Line => ({
    type: 'line',
    tokens: [{ text: '→ ', cls: C.dim }, { text, cls: C.dim }, ...highlights],
    delay,
});

const blank = (delay = 120): Line => ({ type: 'line', tokens: [{ text: '' }], delay });

const check = (
    status: 'pass' | 'slow' | 'fail',
    idx: string,
    name: string,
    label: string,
    timing: string,
    delay = 90,
): Line => {
    const mark = status === 'pass' ? '✓' : status === 'slow' ? '!' : 'x';
    const markCls =
        status === 'pass' ? 'text-term-green/60' : status === 'slow' ? C.yellow : C.red;
    const nameCls = status === 'pass' ? C.dim : C.light;
    const labelCls = status === 'pass' ? '' : status === 'slow' ? C.yellow : C.red;
    const timingCls = status === 'fail' ? C.red : C.dim;
    return {
        type: 'line',
        tokens: [
            { text: `${mark} `, cls: markCls },
            { text: `[${idx}] `, cls: C.dim },
            { text: name, cls: nameCls },
            { text: '  ', cls: C.dim },
            { text: label ? `${label}  ` : '', cls: labelCls },
            { text: timing, cls: timingCls },
        ],
        delay,
    };
};

const failNote = (expected: string, got: string, ref: string, delay = 140): Line => ({
    type: 'line',
    tokens: [
        { text: '  → expected ', cls: C.dim },
        { text: expected, cls: C.green },
        { text: ' drift, got ', cls: C.dim },
        { text: got, cls: C.red },
        { text: ' — see ', cls: C.dim },
        { text: ref, cls: C.cyan },
    ],
    delay,
});

const dotsLine = (text: string, delay = 100): Line => ({
    type: 'line',
    tokens: [{ text: '   ', cls: C.dim }, { text, cls: C.dim }],
    delay,
});

const diffOld = (text: string, delay = 80): Line => ({
    type: 'line',
    tokens: [
        { text: '- ', cls: C.red },
        { text, cls: 'text-term-red/85' },
    ],
    delay,
});

const diffNew = (text: string, delay = 80): Line => ({
    type: 'line',
    tokens: [
        { text: '+ ', cls: C.green },
        { text, cls: 'text-term-green' },
    ],
    delay,
});

const editHeader = (file: string, delay = 200): Line => ({
    type: 'line',
    tokens: [
        { text: '  ', cls: C.dim },
        { text: file, cls: C.cyan },
        { text: '  modified', cls: C.dim },
    ],
    delay,
});

const editSaved = (delay = 250): Line => ({
    type: 'line',
    tokens: [{ text: '  saved.', cls: C.dim }],
    delay,
});

const scenarios: Scenario[] = [
    {
        titlebar: '~/projects/redis-mini · luxctl validate',
        lines: [
            promptLine('aryan', 'lighthouse', '~/projects/redis-mini'),
            cmd('luxctl validate --project redis-mini', 400),
            blank(200),
            arrow('resolving project manifest...'),
            arrow('spinning up isolated runtime ', [{ text: '(rust 1.75)', cls: C.orange }]),
            arrow('running ', [
                { text: '31 checks', cls: C.cyan },
                { text: ' against your solution', cls: C.dim },
            ]),
            blank(),
            check('pass', ' 1/31', 'parses resp simple strings', '', '6ms'),
            check('pass', ' 2/31', 'parses resp bulk strings', '', '4ms'),
            check('pass', ' 3/31', 'handles inline commands', '', '3ms'),
            dotsLine('... 23 more passing'),
            check('slow', '27/31', 'pipelined commands', 'slow', '412ms'),
            check('fail', '28/31', 'expire / ttl precision < 10ms', 'fail', ''),
            failNote('9ms', '47ms', 'notes/expire.md'),
            check('pass', '29/31', 'config get maxmemory', '', '2ms'),
            check('pass', '30/31', 'persistence: aof replay', '', '38ms'),
            check('pass', '31/31', 'concurrent clients (50)', '', '121ms'),
            blank(),
            {
                type: 'summary',
                result: 'fail',
                pass: 29,
                slow: 1,
                fail: 1,
                total: 31,
                delay: 250,
            },
            {
                type: 'tip',
                lead: 'tip: ',
                cmd: 'luxctl explain 28',
                tail: ' for a guided walkthrough of the failing test.',
                delay: 400,
            },
            blank(1400),

            promptLine('aryan', 'lighthouse', '~/projects/redis-mini'),
            cmd('$EDITOR src/expire.go', 400),
            editHeader('src/expire.go'),
            diffOld('if drift > 10 * time.Millisecond {'),
            diffNew('if drift > 5 * time.Microsecond {'),
            editSaved(),
            blank(600),

            promptLine('aryan', 'lighthouse', '~/projects/redis-mini'),
            cmd('luxctl validate --project redis-mini', 350),
            blank(200),
            arrow('running ', [
                { text: '31 checks', cls: C.cyan },
                { text: ' against your solution', cls: C.dim },
            ], 250),
            blank(),
            check('pass', ' 1/31', 'parses resp simple strings', '', '5ms', 50),
            check('pass', ' 2/31', 'parses resp bulk strings', '', '4ms', 50),
            dotsLine('... 25 more passing', 80),
            check('pass', '27/31', 'pipelined commands', '', '118ms', 60),
            check('pass', '28/31', 'expire / ttl precision < 10ms', '', '4ms', 60),
            check('pass', '29/31', 'config get maxmemory', '', '2ms', 50),
            check('pass', '30/31', 'persistence: aof replay', '', '36ms', 50),
            check('pass', '31/31', 'concurrent clients (50)', '', '119ms', 60),
            blank(),
            {
                type: 'summary',
                result: 'pass',
                pass: 31,
                slow: 0,
                fail: 0,
                total: 31,
                delay: 250,
            },
            {
                type: 'verdict',
                result: 'pass',
                grade: 'a',
                xp: 260,
                rank: 142,
                project: 'this project',
                delay: 300,
            },
        ],
    },

    {
        titlebar: '~/projects/tcp-proxy · luxctl validate',
        lines: [
            promptLine('aryan', 'lighthouse', '~/projects/tcp-proxy'),
            cmd('luxctl validate --project tcp-proxy', 400),
            blank(200),
            arrow('resolving project manifest...'),
            arrow('spinning up isolated runtime ', [{ text: '(go 1.23)', cls: C.orange }]),
            arrow('running ', [
                { text: '14 checks', cls: C.cyan },
                { text: ' against your solution', cls: C.dim },
            ]),
            blank(),
            check('pass', ' 1/14', 'parse upstream config', '', '2ms'),
            check('pass', ' 2/14', 'bind listener :8080', '', '3ms'),
            check('pass', ' 3/14', 'accept connection', '', '5ms'),
            check('pass', ' 4/14', 'dial upstream :9000', '', '4ms'),
            dotsLine('... 8 more passing'),
            check('fail', '13/14', 'bidirectional forwarding', 'fail', ''),
            failNote('1024B back', '0B back', 'notes/proxy.md'),
            check('pass', '14/14', 'tcp keepalive set', '', '1ms'),
            blank(),
            {
                type: 'summary',
                result: 'fail',
                pass: 13,
                slow: 0,
                fail: 1,
                total: 14,
                delay: 250,
            },
            {
                type: 'tip',
                lead: 'tip: ',
                cmd: 'luxctl explain 13',
                tail: ' for a guided walkthrough of the failing test.',
                delay: 400,
            },
            blank(1400),

            promptLine('aryan', 'lighthouse', '~/projects/tcp-proxy'),
            cmd('$EDITOR proxy/handler.go', 400),
            editHeader('proxy/handler.go'),
            diffOld('io.Copy(upstream, client)'),
            diffNew('go io.Copy(upstream, client)'),
            diffNew('io.Copy(client, upstream)'),
            editSaved(),
            blank(600),

            promptLine('aryan', 'lighthouse', '~/projects/tcp-proxy'),
            cmd('luxctl validate --project tcp-proxy', 350),
            blank(200),
            arrow('running ', [
                { text: '14 checks', cls: C.cyan },
                { text: ' against your solution', cls: C.dim },
            ], 250),
            blank(),
            check('pass', ' 1/14', 'parse upstream config', '', '2ms', 50),
            dotsLine('... 11 more passing', 80),
            check('pass', '13/14', 'bidirectional forwarding', '', '42ms', 60),
            check('pass', '14/14', 'tcp keepalive set', '', '1ms', 60),
            blank(),
            {
                type: 'summary',
                result: 'pass',
                pass: 14,
                slow: 0,
                fail: 0,
                total: 14,
                delay: 250,
            },
            {
                type: 'verdict',
                result: 'pass',
                grade: 'a',
                xp: 180,
                rank: 87,
                project: 'this project',
                delay: 300,
            },
        ],
    },
];

const currentScenarioIndex = ref(0);
const displayedLines = ref<Line[]>([]);
const isTyping = ref(false);
const currentLineIndex = ref(0);
const currentCharIndex = ref(0);
const terminalContent = ref<HTMLElement | null>(null);

let animationTimeout: ReturnType<typeof setTimeout> | null = null;
let scenarioTimeout: ReturnType<typeof setTimeout> | null = null;

// scenarios is a non-empty literal, so the index is always in range; the
// non-null assertion states that once rather than guarding at every use
const currentScenario = computed<Scenario>(() => scenarios[currentScenarioIndex.value]!);

watch(
    displayedLines,
    () => {
        nextTick(() => {
            if (terminalContent.value) {
                terminalContent.value.scrollTop = terminalContent.value.scrollHeight;
            }
        });
    },
    { deep: true },
);

const processNextLine = (): void => {
    const scenario = currentScenario.value;
    const line = scenario.lines[currentLineIndex.value];

    if (!line) {
        isTyping.value = false;
        scenarioTimeout = setTimeout(() => {
            nextScenario();
        }, 3500);
        return;
    }

    if (line.type === 'line' && line.typed) {
        if (currentCharIndex.value === 0) {
            const seed: Line = {
                type: 'line',
                tokens: line.tokens.map((t, i) => (i === line.tokens.length - 1 ? { ...t, text: '' } : { ...t })),
            };
            displayedLines.value.push(seed);
        }
        typeNextCharacter();
    } else {
        displayedLines.value.push({ ...line });
        currentLineIndex.value++;
        currentCharIndex.value = 0;

        const nextLine = scenario.lines[currentLineIndex.value];
        const delay = nextLine && 'delay' in nextLine ? nextLine.delay || 0 : 0;

        animationTimeout = setTimeout(processNextLine, delay);
    }
};

const typeNextCharacter = (): void => {
    const scenario = currentScenario.value;
    const line = scenario.lines[currentLineIndex.value];
    if (!line || line.type !== 'line') return;

    const targetTok = line.tokens[line.tokens.length - 1];
    const displayed = displayedLines.value[displayedLines.value.length - 1];
    if (!targetTok || !displayed || displayed.type !== 'line') return;

    const target = targetTok.text;
    const lastTok = displayed.tokens[displayed.tokens.length - 1];
    if (!lastTok) return;

    if (currentCharIndex.value < target.length) {
        lastTok.text = target.substring(0, currentCharIndex.value + 1);
        currentCharIndex.value++;

        const baseSpeed = 22;
        const randomDelay = Math.random() * 18;
        animationTimeout = setTimeout(typeNextCharacter, baseSpeed + randomDelay);
    } else {
        currentLineIndex.value++;
        currentCharIndex.value = 0;
        const nextLine = scenario.lines[currentLineIndex.value];
        const delay = nextLine && 'delay' in nextLine ? nextLine.delay || 0 : 0;
        animationTimeout = setTimeout(processNextLine, delay);
    }
};

const startScenario = (): void => {
    displayedLines.value = [];
    currentLineIndex.value = 0;
    currentCharIndex.value = 0;
    isTyping.value = true;
    animationTimeout = setTimeout(processNextLine, 400);
};

const nextScenario = (): void => {
    currentScenarioIndex.value = (currentScenarioIndex.value + 1) % scenarios.length;
    startScenario();
};

const goToScenario = (index: number): void => {
    if (animationTimeout) clearTimeout(animationTimeout);
    if (scenarioTimeout) clearTimeout(scenarioTimeout);
    currentScenarioIndex.value = index;
    startScenario();
};

onMounted(() => {
    startScenario();
});

onUnmounted(() => {
    if (animationTimeout) clearTimeout(animationTimeout);
    if (scenarioTimeout) clearTimeout(scenarioTimeout);
});
</script>

<template>
    <div
        class="terminal-window overflow-hidden rounded-xl bg-term-bg"
        style="
            box-shadow:
                0 25px 50px -12px rgba(0, 0, 0, 0.5),
                0 40px 80px -20px rgba(0, 0, 0, 0.35);
        "
    >
        <!-- Content -->
        <div
            ref="terminalContent"
            class="terminal-content h-[520px] overflow-y-auto bg-term-bg px-6 py-5 font-mono text-[13px] leading-[1.75]"
        >
            <template v-for="(line, index) in displayedLines" :key="index">
                <!-- token line -->
                <div v-if="line.type === 'line'" class="whitespace-pre-wrap">
                    <span
                        v-for="(tok, ti) in line.tokens"
                        :key="ti"
                        :class="tok.cls || 'text-term-text'"
                        >{{ tok.text }}</span
                    >
                    <span
                        v-if="
                            index === displayedLines.length - 1 &&
                            isTyping &&
                            line.tokens.some((t) => t.text.length > 0)
                        "
                        class="cursor animate-pulse text-term-green"
                        >▋</span
                    >
                </div>

                <!-- summary -->
                <div v-else-if="line.type === 'summary'" class="mt-1 flex flex-wrap items-center gap-x-1 whitespace-pre-wrap">
                    <span class="text-term-text">summary</span>
                    <span class="text-term-dim">·</span>
                    <span class="text-term-green">{{ line.pass }} pass</span>
                    <span v-if="line.slow > 0" class="text-term-dim">·</span>
                    <span v-if="line.slow > 0" class="text-term-yellow">{{ line.slow }} slow</span>
                    <span v-if="line.fail > 0" class="text-term-dim">·</span>
                    <span v-if="line.fail > 0" class="text-term-red">{{ line.fail }} fail</span>
                    <span class="text-term-dim">·</span>
                    <span class="text-term-dim">{{ line.total }} total</span>
                </div>

                <!-- verdict (pass badge + xp + rank) -->
                <div v-else-if="line.type === 'verdict'" class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 whitespace-pre-wrap">
                    <span class="font-semibold tracking-wide text-term-green">{{ line.result }}</span>
                    <span v-if="line.grade" class="text-term-dim">
                        grade: <span class="text-term-text">{{ line.grade }}</span>
                    </span>
                    <span v-if="line.xp" class="font-semibold text-term-yellow">+{{ line.xp }} xp</span>
                    <span v-if="line.rank" class="text-term-dim"
                        >rank <span class="text-term-text">#{{ line.rank }}</span> on {{ line.project }}</span
                    >
                </div>

                <!-- tip -->
                <div v-else-if="line.type === 'tip'" class="whitespace-pre-wrap">
                    <span class="text-term-dim">{{ line.lead }}</span>
                    <span class="text-term-blue">{{ line.cmd }}</span>
                    <span class="text-term-dim">{{ line.tail }}</span>
                </div>
            </template>

            <div
                v-if="!isTyping || displayedLines.length === 0"
                class="text-term-green"
            >
                <span class="text-term-dim">$ </span>
                <span class="cursor animate-pulse">▋</span>
            </div>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-end bg-term-void px-4 py-3">
            <div class="flex gap-2">
                <button
                    v-for="(_, index) in scenarios"
                    :key="index"
                    type="button"
                    class="size-2 rounded-full transition-all duration-300"
                    :class="
                        currentScenarioIndex === index
                            ? 'scale-125 bg-term-green'
                            : 'bg-term-line hover:bg-term-gray'
                    "
                    :aria-label="`Go to scenario ${index + 1}`"
                    @click="goToScenario(index)"
                />
            </div>
        </div>
    </div>
</template>

<style scoped>
.terminal-window {
    font-family: 'JetBrains Mono', 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
}

.terminal-content {
    scrollbar-width: thin;
    scrollbar-color: var(--color-term-panel) transparent;
}

.terminal-content::-webkit-scrollbar {
    width: 8px;
}

.terminal-content::-webkit-scrollbar-track {
    background: var(--color-term-bg);
}

.terminal-content::-webkit-scrollbar-thumb {
    background-color: var(--color-term-panel);
    border-radius: 4px;
}

.terminal-content::-webkit-scrollbar-thumb:hover {
    background-color: var(--color-term-line);
}

@keyframes pulse {
    0%,
    100% {
        opacity: 1;
    }
    50% {
        opacity: 0;
    }
}
</style>
