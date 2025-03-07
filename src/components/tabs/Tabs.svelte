<script lang="ts">
  import type { DiffFilepaths, StartupParam } from '../../types'
  import FileHandle from './file-handle/FileHandle.svelte'
  import DragDrop from '../common/DragDrop.svelte'
  import AppTab from './AppTab.svelte'
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'

  let diffFilepathsList: (DiffFilepaths | null)[] = $state([null])
  let activeTabIndex: number = $state(0)

  let showsFileHandle: boolean = $state(false)

  let fileHandleOldFilepath: string = $state('')
  let fileHandleNewFilepath: string = $state('')

  const filepathsOnChange = (diffFilepaths: DiffFilepaths) => {
    addDiffTab(diffFilepaths)
    showsFileHandle = false
  }

  const addDiffTab = (diffFilepaths: DiffFilepaths) => {
    diffFilepathsList.push(diffFilepaths)
    activeTabIndex = diffFilepathsList.length - 1
  }

  const filesOnDropped = (filepaths: string[]) => {
    if (filepaths.length === 0) return

    // open file handle
    if (filepaths.length === 1) {
      if (0 < fileHandleOldFilepath.length) {
        fileHandleNewFilepath = filepaths[0]
      } else {
        fileHandleOldFilepath = filepaths[0]
        fileHandleNewFilepath = ''
      }
      showsFileHandle = true
      return
    }

    // show diff directly
    const diffFilepaths = { old: filepaths[0], new: filepaths[1] } as DiffFilepaths
    diffFilepathsList.push(diffFilepaths)
    activeTabIndex = diffFilepathsList.length - 1
  }

  const removeTab = (tabIndex: number) => {
    if (tabIndex === activeTabIndex) {
      activeTabIndex -= 1
    }
    diffFilepathsList.splice(tabIndex, 1)
  }

  onMount(async () => {
    const res = (await invoke('ready').catch((error: unknown) => {
      console.error(error)
      return
    })) as StartupParam

    if (res.oldFilepath) {
      if (res.newFilepath) {
        // show startup diff tab
        addDiffTab({
          old: res.oldFilepath,
          new: res.newFilepath,
        } as DiffFilepaths)
      } else {
        // start with a file dropped
        filesOnDropped([res.oldFilepath])
      }
    }
  })
</script>

<button class="shows-file-handle" onclick={() => (showsFileHandle = !showsFileHandle)}>+</button>

<div class="tabs">
  <div class="headers">
    {#each diffFilepathsList as _diffFilepaths, tabIndex}
      <div class={`header ${tabIndex === activeTabIndex ? 'active' : ''}`}>
        <label><input type="radio" value={tabIndex} bind:group={activeTabIndex} />{tabIndex}</label>
        {#if 0 < tabIndex}
          <button onclick={() => removeTab(tabIndex)}>x</button>
        {/if}
      </div>
    {/each}
  </div>
</div>
<div class="active-tab">
  {#each diffFilepathsList as diffFilepaths, tabIndex}
    <div class={tabIndex === activeTabIndex ? '' : 'd-none'}>
      <AppTab
        {diffFilepaths}
        diffFilepathsOnSelected={addDiffTab}
        removeDiffTab={() => removeTab(tabIndex)}
      />
    </div>
  {/each}
</div>

<div class="drag-drop">
  <DragDrop onDrop={filesOnDropped} />
</div>

<div class={`select-files ${showsFileHandle ? '' : 'd-none'}`}>
  <button onclick={() => (showsFileHandle = !showsFileHandle)}>x</button>
  {#key [fileHandleOldFilepath, fileHandleNewFilepath]}
    <FileHandle
      oldFilepath={fileHandleOldFilepath}
      newFilepath={fileHandleNewFilepath}
      {filepathsOnChange}
    />
  {/key}
</div>

<style>
  .drag-drop {
    position: fixed;
    left: 0;
    top: 0;
    width: 100vw;
    height: 100vh;
    z-index: 0;
    pointer-events: none;
  }

  .select-files {
    position: fixed;
    left: 10vw;
    top: 10vh;
    width: 80vw;
    height: 80vh;
    padding: 0.4rem 0;
    background-color: yellow;
    color: #252525;
  }

  input[type='radio'] {
    display: none;
  }

  .headers {
    max-width: 100%;
    display: flex;
    overflow-x: auto;
  }
  .header {
    width: 5.7rem;
    display: flex;
    align-items: center;
    border: 0.01rem solid yellow;
  }
  .header.active {
    font-size: 110%;
    border-color: coral;
    border-width: 0.32rem;
  }

  .header button {
    width: 1.4rem;
    padding: 0.1rem 0.4rem;
    margin: 0 0.3rem;
    flex-grow: 0;
  }
  .header label {
    width: 100%;
    display: block;
    flex: 1;
    text-align: center;
  }
  .header label:hover {
    opacity: 0.6;
  }

  .shows-file-handle {
    padding: 0.5rem 1.5rem;
  }
</style>
