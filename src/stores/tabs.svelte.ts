import { get, writable, type Writable } from "svelte/store";
import type { CompareSet } from "../types";
import { binaryComparisonOnly } from "../utils/diff.svelte";

let activeCompareSetIndex: number | null = $state(null)

export const compareSets: Writable<CompareSet[]> = writable([])

export const pushCompareSet = (compareSet: CompareSet) => {
    compareSets.update((x) => { x.push(compareSet); return x })
    activeCompareSetIndex = get(compareSets).length - 1
}

export const spliceCompareSet = (index: number) => {
    compareSets.update((x) => { x.splice(index, 1); return x })

    if (get(compareSets).length === 0) {
        activeCompareSetIndex = null
        return
    }

    const activeCompareSetIndexIsPreserved =
        activeCompareSetIndex === null ||
        activeCompareSetIndex === 0 ||
        activeCompareSetIndex < index
    if (!activeCompareSetIndexIsPreserved) {
        activeCompareSetIndex! -= 1
    }
}

export const getActiveCompareSet = (): CompareSet | null => {
    if (activeCompareSetIndex === null) return null
    return get(compareSets)[activeCompareSetIndex]
}

export const updateActiveCompareSet = async (oldFilepath: string, newFilepath: string) => {
    const activeCompareSet = getActiveCompareSet()!
    activeCompareSet.old.filepath = oldFilepath
    activeCompareSet.old.binaryComparisonOnly = await binaryComparisonOnly(oldFilepath)
    activeCompareSet.new.filepath = newFilepath
    activeCompareSet.new.binaryComparisonOnly = await binaryComparisonOnly(newFilepath)
}

export const isActiveCompareSetIndex = (index: number | null): boolean => {
    return index === activeCompareSetIndex
}

export const activateCompareSet = (index: number) => {
    activeCompareSetIndex = index
}

export const removeActiveCompareSet = () => {
    if (activeCompareSetIndex === null) return

    spliceCompareSet(activeCompareSetIndex)
    if (activeCompareSetIndex !== null) {
        // ask svelte to update view
        const i = activeCompareSetIndex
        activeCompareSetIndex = null
        activeCompareSetIndex = i
    }
}

export const activateExplorer = () => {
    activeCompareSetIndex = null
}

export const isActiveCompareSet = (index: number | null) => {
    return index !== null && index === activeCompareSetIndex
}

export const exploreIsActive = (): boolean => {
    return activeCompareSetIndex === null
}