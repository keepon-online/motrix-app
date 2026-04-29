import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'

const visibilityModule = await import('../src-vue/utils/taskVisibility.ts')

test('new tasks switch stopped and waiting views back to downloading when enabled', () => {
  assert.equal(
    visibilityModule.getPostAddTaskListType('stopped', true),
    'active',
  )
  assert.equal(
    visibilityModule.getPostAddTaskListType('waiting', true),
    'active',
  )
})

test('new tasks keep the current list when show-downloading is disabled', () => {
  assert.equal(
    visibilityModule.getPostAddTaskListType('stopped', false),
    'stopped',
  )
  assert.equal(
    visibilityModule.getPostAddTaskListType('waiting', false),
    'waiting',
  )
})

test('show-downloading defaults to enabled when config is absent', () => {
  assert.equal(
    visibilityModule.shouldShowDownloadingAfterAdd(undefined),
    true,
  )
})

test('add task flows do not navigate while submitting', async () => {
  const files = [
    'src-vue/components/AddTaskDialog.vue',
    'src-vue/components/DragDrop.vue',
  ]

  for (const file of files) {
    const source = await readFile(file, 'utf8')
    assert.doesNotMatch(source, /router\.(push|replace)\(/)
  }
})
