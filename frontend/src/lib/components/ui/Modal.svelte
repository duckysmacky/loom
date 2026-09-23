<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		open,
		onclose,
		label,
		width = 460,
		children
	}: {
		open: boolean;
		onclose: () => void;
		label: string;
		width?: number;
		children: Snippet;
	} = $props();

	let dialog: HTMLDialogElement | undefined = $state();

	$effect(() => {
		if (!dialog) return;
		if (open && !dialog.open) dialog.showModal();
		if (!open && dialog.open) dialog.close();
	});
</script>

<!-- Native <dialog>: focus trap, Esc and inert background come for free. -->
<dialog
	bind:this={dialog}
	aria-label={label}
	style:width="min({width}px, calc(100vw - 32px))"
	{onclose}
	onclick={(event) => event.target === dialog && onclose()}
>
	{#if open}{@render children()}{/if}
</dialog>

<style>
	dialog {
		padding: 0;
		border: var(--border-width) solid var(--ink);
		background: var(--surface);
		color: var(--ink);
		margin-top: 12vh;
	}

	dialog::backdrop {
		background: var(--scrim);
	}
</style>
