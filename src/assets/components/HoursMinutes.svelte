<script lang="ts">
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    /// Input value in minutes
    export let value = 0;

    let inputEl: HTMLInputElement | null = null;

    function setInput(inputEl: HTMLInputElement, minutes: number) {
        inputEl.valueAsNumber = minutes * 60 * 1000;
    }

    $: if (inputEl) setInput(inputEl, value);

    function inputChanged(e: Event) {
        let element = e.target as HTMLInputElement;
        const newValueMinutes = element.valueAsNumber / 1000 / 60;
        if (value != newValueMinutes)
            value = newValueMinutes;

        dispatch('change', e);
    }

    export function focus() {
        if (inputEl) inputEl.focus();
    }
</script>

<input type="time" bind:this={inputEl} on:input={inputChanged} max="08:00:00" min="00:00:00"
       pattern="0[0-8]:[0-9][0-9]" class="form-control"/>