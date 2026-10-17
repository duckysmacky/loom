<script lang="ts">
	import { NodeResizer, type NodeProps } from '@xyflow/svelte';
	import SideHandles from '$lib/components/canvas/SideHandles.svelte';
	import { untrack } from 'svelte';
	import { canvasApi } from '$lib/api/endpoints';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import type { LoomNoteNode } from '$lib/components/canvas/types';

	// Same minimum the API enforces.
	const NOTE_MIN_WIDTH = 80;
	const NOTE_MIN_HEIGHT = 50;

	let { data, selected }: NodeProps<LoomNoteNode> = $props();

	const note = $derived(data.note);
	// A fresh (empty) note opens ready to type.
	let editing = $state(untrack(() => data.note.body === ''));
	let draft = $state('');

	function startEditing() {
		draft = note.body;
		editing = true;
	}

	async function save(changes: Parameters<typeof canvasApi.updateNote>[1]) {
		try {
			graph.replaceNote(await canvasApi.updateNote(note.id, changes));
		} catch (error) {
			notifyError(error);
		}
	}

	function finishEditing() {
		if (!editing) return;
		editing = false;
		if (draft !== note.body) save({ body: draft });
	}

	async function remove() {
		const removed = await graph.mutate(() => canvasApi.removeNote(note.id));
		if (removed !== undefined) notify('Note deleted');
	}

	// The textarea grows with its text so the flex box can centre it.
	const fitHeight = (element: HTMLTextAreaElement) => {
		element.style.height = 'auto';
		element.style.height = `${element.scrollHeight}px`;
	};

	const focus = (element: HTMLTextAreaElement) => {
		fitHeight(element);
		element.focus();
		element.setSelectionRange(element.value.length, element.value.length);
	};
</script>

<NodeResizer
	isVisible={selected && !editing}
	minWidth={NOTE_MIN_WIDTH}
	minHeight={NOTE_MIN_HEIGHT}
	color="var(--note-edge)"
	onResizeEnd={(_, params) =>
		save({
			canvas_x: Math.round(params.x),
			canvas_y: Math.round(params.y),
			canvas_width: Math.round(params.width),
			canvas_height: Math.round(params.height)
		})}
/>
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="sticky" class:selected ondblclick={startEditing}>
	{#if editing}
		<textarea
			class="nodrag nowheel"
			aria-label="Note text"
			placeholder="Type a note…"
			rows="1"
			bind:value={draft}
			use:focus
			oninput={(event) => fitHeight(event.currentTarget)}
			onblur={finishEditing}
			onkeydown={(event) => {
				if (event.key === 'Escape') {
					draft = note.body;
					event.currentTarget.blur();
				}
			}}></textarea>
	{:else}
		<div class="body" class:placeholder={!note.body}>{note.body || 'Double-click to write'}</div>
	{/if}
	<button
		type="button"
		class="remove nodrag"
		aria-label="Delete note"
		title="Delete note"
		onclick={remove}>×</button
	>
</div>
<SideHandles />

<style>
	/* Same frame as a card: thick border, heavier top edge. Text sits in
	   the middle, scrolling once it outgrows the note. */
	.sticky {
		position: relative;
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		justify-content: safe center;
		padding: 10px 12px;
		overflow: auto;
		background: var(--note-fill);
		border: var(--border-width) solid var(--note-edge);
		border-top-width: 5px;
		color: var(--note-ink);
	}

	.selected {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}

	.body,
	textarea {
		width: 100%;
		flex: none;
		text-align: center;
		font: 500 13.5px/1.4 var(--font-display);
		color: inherit;
	}

	.body {
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}

	.placeholder {
		opacity: 0.55;
	}

	textarea {
		display: block;
		resize: none;
		overflow: hidden;
		padding: 0;
		border: 0;
		background: transparent;
		outline: none;
	}

	.remove {
		position: absolute;
		top: 2px;
		right: 4px;
		border: 0;
		background: none;
		color: inherit;
		font: 700 15px/1 var(--font-mono);
		opacity: 0;
		cursor: pointer;
	}

	.sticky:hover .remove,
	.remove:focus-visible {
		opacity: 0.6;
	}

	.remove:hover {
		opacity: 1;
	}
</style>
