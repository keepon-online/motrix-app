export type TaskListType = 'active' | 'waiting' | 'stopped'

export function shouldShowDownloadingAfterAdd(newTaskShowDownloading?: boolean): boolean {
  return newTaskShowDownloading ?? true
}

export function getPostAddTaskListType(
  currentListType: TaskListType,
  newTaskShowDownloading?: boolean,
): TaskListType {
  return shouldShowDownloadingAfterAdd(newTaskShowDownloading)
    ? 'active'
    : currentListType
}
