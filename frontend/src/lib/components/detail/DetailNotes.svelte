<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import { renderMarkdown } from '$lib/markdown';
	import { graph } from '$lib/stores/graph.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	let { node }: { node: NodeResponse } = $props();

	let editing = $state(false);
	let draft = $state('');

	function startEditing() {
		draft = node.notes ?? '';
		editing = true;
	}

	async function save() {
		const notes = draft.trim() ? draft : null;
		const saved = await graph.mutate(() => nodesApi.update(node.id, { notes }));
		if (saved) editing = false;
	}
</script>

<section>
	<div class="head">
		<span class="label">Description</span>
		{#if !editing}
			<button type="button" class="link" onclick={startEditing}>Edit markdown</button>
		{/if}
	</div>

	{#if editing}
		<textarea
			class="field editor"
			rows="8"
			aria-label="Notes (markdown)"
			bind:value={draft}
			onkeydown={(event) => {
				if (event.key === 'Enter' && event.ctrlKey) save();
				if (event.key === 'Escape') {
					event.stopPropagation();
					editing = false;
				}
			}}></textarea>
		<div class="actions">
			<span class="hint">Ctrl ↵ to save</span>
			<Button variant="quiet" onclick={() => (editing = false)}>Cancel</Button>
			<Button variant="primary" onclick={save}>Save</Button>
		</div>
	{:else if node.notes}
		<!-- Sanitized by DOMPurify in renderMarkdown. -->
		<!-- eslint-disable-next-line svelte/no-at-html-tags -->
		<div class="markdown">{@html renderMarkdown(node.notes)}</div>
	{:else}
		<button type="button" class="placeholder" onclick={startEditing}>
			No description yet. Add notes, links, a plan…
		</button>
	{/if}
</section>

<style>
	.head {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.link {
		margin-left: auto;
		border: none;
		background: none;
		padding: 0;
		font: 700 11px/1 var(--font-display);
		color: var(--accent);
	}

	.editor {
		margin-top: 10px;
		resize: vertical;
		font: 400 12.5px/1.55 var(--font-mono);
	}

	.actions {
		margin-top: 8px;
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
	}

	.hint {
		margin-right: auto;
		font: 500 10.5px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.markdown {
		margin-top: 10px;
		font: 500 13.5px/1.6 var(--font-display);
		color: var(--ink-3);
		overflow-wrap: anywhere;
	}

	.markdown :global(:first-child) {
		margin-top: 0;
	}

	.markdown :global(h1),
	.markdown :global(h2),
	.markdown :global(h3) {
		margin: 14px 0 6px;
		font: 700 14.5px/1.3 var(--font-display);
		color: var(--ink);
	}

	.markdown :global(p),
	.markdown :global(ul),
	.markdown :global(ol) {
		margin: 6px 0;
	}

	.markdown :global(ul),
	.markdown :global(ol) {
		padding-left: 20px;
	}

	.markdown :global(code) {
		font: 400 12px/1.4 var(--font-mono);
		background: var(--surface-2);
		padding: 1px 4px;
	}

	.markdown :global(pre) {
		background: var(--surface-2);
		border: var(--border-width-hair) solid var(--line);
		padding: 10px;
		overflow-x: auto;
	}

	.markdown :global(pre code) {
		background: none;
		padding: 0;
	}

	.markdown :global(blockquote) {
		margin: 6px 0;
		padding-left: 10px;
		border-left: var(--border-width) solid var(--line);
	}

	.placeholder {
		margin-top: 10px;
		width: 100%;
		text-align: left;
		padding: 12px;
		border: var(--border-width-hair) dashed var(--line);
		background: none;
		font: 500 12.5px/1.4 var(--font-display);
		color: var(--ink-2);
	}
</style>
