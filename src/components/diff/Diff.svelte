<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import type {
    CompareSet,
    DiffResponse,
    LinesDiff,
    OldOrNew,
    CharsDiffLines,
    CharsDiffResponse,
  } from '../../types'
  import { DIFF_LINE_HEIGHT } from './consts'
  import DiffCol from './diff-col/DiffCol.svelte'
  import DiffHeaderCol from './diff-col/DiffHeaderCol.svelte'
  import DiffFooterCol from './diff-col/DiffFooterCol.svelte'
  import SeparatorCol from './separator-col/SeparatorCol.svelte'
  import SeparatorHeaderCol from './separator-col/SeparatorHeaderCol.svelte'
  import SeparatorFooterCol from './separator-col/SeparatorFooterCol.svelte'
  import { openFileDialog, saveFileDialog } from '../../utils/dialog.svelte'
  import { onMount } from 'svelte'
  import { errorToast } from '../../stores/Toast.svelte'
  import {
    getActiveCompareSet,
    removeActiveCompareSet,
    updateActiveCompareSet,
  } from '../../stores/tabs.svelte'

  const { compareSet }: { compareSet: CompareSet } = $props()

  let _compareSet: CompareSet = $state(compareSet)

  const oldFilepath: string = $derived(_compareSet.old.filepath)
  const newFilepath: string = $derived(_compareSet.new.filepath)

  let oldCharset: string = $state('')
  let newCharset: string = $state('')

  let linesDiffs: LinesDiff[] = $state([])
  let charsDiffs: CharsDiffLines[] | null = $state(null)
  let showsCharsDiffs: boolean = $state(false)
  let focusedLinesDiffIndex: number | null = $state(null)

  let loaded: boolean = $state(false)

  let diffPanes: HTMLDivElement

  onMount(async () => {
    await diff()
    diffPanes.focus()
  })

  // todo: not equal diffs in linesDiffs can changed to be 'equal'
  // const linesDiffsDiffOnly: number[] = $derived(
  //   linesDiffs
  //     .map((x, i) => (x.diffKind !== 'equal' ? i : undefined))
  //     .filter((x) => x !== undefined)
  // )
  const prevLinesDiffIndex: number | null = $derived.by(() => {
    if (isCompletelyEqual) return null
    if (focusedLinesDiffIndex === null) return 0
    if (focusedLinesDiffIndex === 0) return focusedLinesDiffIndex
    const foundIndex = linesDiffs.findLastIndex(
      (x, i) => i < focusedLinesDiffIndex! && x.diffKind !== 'equal'
    )
    return 0 <= foundIndex ? foundIndex : focusedLinesDiffIndex
  })
  const nextLinesDiffIndex: number | null = $derived.by(() => {
    if (isCompletelyEqual) return null
    if (focusedLinesDiffIndex === null) return 0
    if (focusedLinesDiffIndex === linesDiffs.length - 1) return focusedLinesDiffIndex
    const foundIndex = linesDiffs.findIndex(
      (x, i) => focusedLinesDiffIndex! < i && x.diffKind !== 'equal'
    )
    return 0 <= foundIndex ? foundIndex : focusedLinesDiffIndex
  })

  const charsDiffsAvailable: boolean = $derived(
    oldFilepath !== newFilepath &&
      !compareSet!.old.binaryComparisonOnly &&
      !compareSet!.new.binaryComparisonOnly
  )

  const switchOldNewAvailable: boolean = $derived(oldFilepath !== newFilepath)

  const isCompletelyEqual: boolean = $derived(!linesDiffs.some((x) => x.diffKind !== 'equal'))

  const diff = async () => {
    await diffLines()
    loaded = true
  }

  const diffLines = async () => {
    let isError = false
    let res: unknown = await invoke('diff_filepaths', {
      old: oldFilepath,
      new: newFilepath,
    }).catch((error: unknown) => {
      errorToast(error as string)
      isError = true
    })
    if (isError) {
      removeActiveCompareSet()
    }

    console.log(res) // todo

    const diffResponse = res as DiffResponse
    linesDiffs = diffResponse.linesDiffs
    oldCharset = diffResponse.oldCharset
    newCharset = diffResponse.newCharset

    focusedLinesDiffIndex = null
  }

  const diffChars = async () => {
    const replaceLinesDiffs = linesDiffs.filter((x) => x.diffKind === 'replace')
    if (replaceLinesDiffs.length === 0) return

    let res: unknown = await invoke('diff_chars', {
      linesDiffs: replaceLinesDiffs,
    }).catch((error: unknown) => {
      console.error(error)
      return
    })
    console.log(res) // todo

    const charsDiffResponse = res as CharsDiffResponse
    charsDiffs = charsDiffResponse.diffs
  }

  const changeFilepath = async (oldOrNew: OldOrNew) => {
    const filepath = await openFileDialog()
    if (filepath === null) return
    if (oldOrNew === 'old') {
      const _oldFilepath = filepath
      await updateActiveCompareSet(_oldFilepath, newFilepath)
    } else {
      const _newFilepath = filepath
      await updateActiveCompareSet(oldFilepath, _newFilepath)
    }
    _compareSet = getActiveCompareSet()!
    await diff()
  }

  const linesDiffReplaceOnClick = (linesDiffIndex: number) => {
    const x = linesDiffs[linesDiffIndex]
    x.diffKind = 'equal'
    x.newLines = x.oldLines
    linesDiffs[linesDiffIndex] = x
    if (focusedLinesDiffIndex === linesDiffIndex) focusedLinesDiffIndex = null
  }

  const saveAsOnClick = async () => {
    const filepath = await saveFileDialog(newFilepath!).catch((error: unknown) => {
      console.error(error)
      return
    })
    if (!filepath) return
    await invoke('save', {
      filepath: filepath,
      content: linesDiffs.reduce((a, b) => `${a}${b.newLines.join('')}`, ''),
      charset: newCharset,
    }).catch((error: unknown) => {
      console.error(error)
      return
    })
  }

  const showsCharsDiffsOnChange = async (value: boolean) => {
    if (charsDiffs === null) {
      await diffChars()
    }
    showsCharsDiffs = value
  }

  const switchOldNewOnClick = async () => {
    await updateActiveCompareSet(newFilepath, oldFilepath)
    _compareSet = getActiveCompareSet()!

    linesDiffs = linesDiffs.map((x) => {
      const ret = x
      const orgOldLines = ret.oldLines
      ret.oldLines = ret.newLines
      ret.newLines = orgOldLines
      return ret
    })

    const orgOldCharset = oldCharset
    oldCharset = newCharset
    newCharset = orgOldCharset

    charsDiffs = charsDiffs!.map((x) => {
      const ret = x
      const orgOldLines = ret.oldLines
      ret.oldLines = ret.newLines
      ret.newLines = orgOldLines
      return ret
    })

    focusedLinesDiffIndex = null
  }

  const onKeyDown = (
    e: KeyboardEvent & {
      currentTarget: EventTarget & HTMLDivElement
    }
  ) => {
    switch (e.key) {
      case 'w': {
        if (e.ctrlKey) {
          removeActiveCompareSet()
        }
      }
      case 'F7': {
        focusedLinesDiffIndex = prevLinesDiffIndex
        break
      }
      case 'F8': {
        focusedLinesDiffIndex = nextLinesDiffIndex
        break
      }
      default:
    }
  }
</script>

<div class="diff-panes" onkeydown={onKeyDown} role="button" tabindex="0" bind:this={diffPanes}>
  {#if !loaded}<p>(...... Loading ......)</p>{/if}

  {#if 0 < linesDiffs.length}
    <div class="rows">
      <div class="row header">
        <div class="col diff old">
          <DiffHeaderCol
            filepath={oldFilepath!}
            filepathFromDialogOnClick={async () => changeFilepath('old')}
          />
        </div>
        <div class="col separator">
          <SeparatorHeaderCol
            {showsCharsDiffs}
            {charsDiffsAvailable}
            {switchOldNewAvailable}
            {showsCharsDiffsOnChange}
            {switchOldNewOnClick}
          />
        </div>
        <div class="col diff new">
          <DiffHeaderCol
            filepath={newFilepath!}
            filepathFromDialogOnClick={async () => changeFilepath('new')}
          />
        </div>
      </div>
      <div class="row content" style={`--line-height: ${DIFF_LINE_HEIGHT};`}>
        <div class="col diff old">
          {#key focusedLinesDiffIndex}
            <DiffCol
              oldOrNew="old"
              {linesDiffs}
              {charsDiffs}
              {showsCharsDiffs}
              {focusedLinesDiffIndex}
            />
          {/key}
        </div>
        <div class="col separator">
          <SeparatorCol
            {linesDiffs}
            {focusedLinesDiffIndex}
            replaceOnClick={linesDiffReplaceOnClick}
          />
        </div>
        <div class="col diff new">
          {#key focusedLinesDiffIndex}
            <DiffCol
              oldOrNew="new"
              {linesDiffs}
              {charsDiffs}
              {showsCharsDiffs}
              {focusedLinesDiffIndex}
            />
          {/key}
        </div>
      </div>
      <div class="row footer">
        <div class="col diff old">
          <DiffFooterCol charset={oldCharset} {isCompletelyEqual} saveAsOnClick={undefined} />
        </div>
        <div class="col separator">
          <SeparatorFooterCol />
        </div>
        <div class="col diff new">
          <DiffFooterCol charset={newCharset} {isCompletelyEqual} {saveAsOnClick} />
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .header {
    height: 2rem;
    gap: 1.1rem;
  }

  .rows {
    height: calc(100vh - 1.9rem);
    display: flex;
    flex-direction: column;
  }

  .row.header,
  .row.footer {
    flex-grow: 0;
  }

  .row.header {
    height: 2.7rem;
  }
  .row.header .col {
    overflow-x: auto;
  }

  .row.content {
    height: 100%;
    overflow-y: auto;
  }
  .row.content .col {
    height: fit-content;
    min-height: 100%;
    overflow-y: hidden;
  }

  .row.footer {
    height: 1.5rem;
  }

  .col.separator {
    flex-grow: 0;
    flex-basis: 1.4rem;
  }
</style>
