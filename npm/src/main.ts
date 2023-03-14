#!/usr/bin/env node

import { spawnSync } from 'child_process'
import { chmodSync } from 'fs'

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
  const exePath = getExePath()

  chmodSync(exePath, 0o755)
  const processResult = spawnSync(exePath, args, { stdio: 'inherit' })

  if (processResult.error) {
    throw processResult.error
  }

  process.exit(processResult.status ?? 0)
}

runPalinter()
