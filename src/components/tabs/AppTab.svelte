<script lang="ts">
  import type { DiffFilepaths } from '../../types'
  import Diff from '../diff/Diff.svelte'
  import Explorer from '../explorer/Explorer.svelte'

  const {
    diffFilepaths,
    diffFilepathsOnSelected,
    removeDiffTab,
  }: {
    diffFilepaths: DiffFilepaths | null
    diffFilepathsOnSelected: (diffFilepaths: DiffFilepaths) => void
    removeDiffTab: () => void
  } = $props()
</script>

{#if diffFilepaths === null}
  <Explorer {diffFilepathsOnSelected} />
{:else}
  <Diff {diffFilepaths} {removeDiffTab} />
{/if}
