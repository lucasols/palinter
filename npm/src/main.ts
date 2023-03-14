#!/usr/bin/env node

import { spawnSync } from 'child_process'

function getExePath() {
  try {
    // Since the bin will be located inside `node_modules`, we can simply call require.resolve
    return require.resolve(`../bin/palinter`)
  } catch (e) {
    throw new Error(`Couldn't find binary`)
  }
}

function runPalinter() {
  const args = process.argv.slice(2)
  const processResult = spawnSync(getExePath(), args, { stdio: 'inherit' })
  process.exit(processResult.status ?? 0)
}

runPalinter()
