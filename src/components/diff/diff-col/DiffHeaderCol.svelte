<script lang="ts">
  import type { OldOrNew } from '../../../types'

  const {
    oldOrNew,
    filepath,
    isCompletelyEqual,
    filepathFromDialogOnClick,
  }: {
    oldOrNew: OldOrNew
    filepath: string
    isCompletelyEqual: boolean
    filepathFromDialogOnClick: () => void
  } = $props()
</script>

<button onclick={filepathFromDialogOnClick}>
  <h3>{oldOrNew.toUpperCase()}</h3>
  <div>
    {filepath}
  </div>
</button>

{#if isCompletelyEqual}
  <!-- todo: The same files -->
  <div>The files are the same</div>
{/if}

<style>
  button {
    width: 100%;
    padding: 0.3rem 0.4rem;
    display: flex;
    align-items: center;
  }

  h3 {
    padding: 0;
    margin: 0 1rem 0 0;
    display: inline-block;
    font-size: 0.8rem;
    font-weight: normal;
  }

  button div {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: right;
    font-size: 1.05rem;
    font-weight: bold;
  }
</style>
