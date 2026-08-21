#!/usr/bin/env node
// Collects the numbers the shell measured into one JSON file for the report.
// Node rather than python: the toolchain here is javascript and shell, and a
// third language in the deploy path is a third thing to install on a fresh box.
import { readFileSync, writeFileSync } from 'node:fs'

const DIR = 'bench/results'
const num = (name, fallback = 0) => Number(process.env[name] ?? fallback)

const firstLoad = readFileSync(`${DIR}/first-load.csv`, 'utf8')
  .split('\n')
  .filter(Boolean)
  .map((line) => {
    const [path, ttfb, total, bytes] = line.split(',')
    return { path, ttfb: Number(ttfb), total: Number(total), bytes: Number(bytes) }
  })

writeFileSync(`${DIR}/meta.json`, `${JSON.stringify({
  generated: new Date().toLocaleString('en-GB', { dateStyle: 'medium', timeStyle: 'short' }),
  image: process.env.IMAGE ?? 'lighthouse-web:local',
  imageSize: process.env.IMAGE_SIZE ?? '',
  bootMs: num('BOOT_MS'),
  cpuAvg: num('CPU_AVG'),
  cpuMax: num('CPU_MAX'),
  memAvg: num('MEM_AVG'),
  memMax: num('MEM_MAX'),
  firstLoad,
}, null, 2)}\n`)
