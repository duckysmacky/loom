<script lang="ts">
	import { ScrollText } from '@lucide/svelte';
	import Icon from '$lib/components/ui/Icon.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import { shortDate, toTimeInput } from '$lib/graph/display';
	import { openNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { notifyError } from '$lib/stores/toasts.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { PokeResponse } from '$lib/types/PokeResponse';

	/**
	 * Small square button beside the node view's Poke buttons: opens the node's
	 * poke history, newest first. A path's history is every poke inside it, so
	 * each entry names the node it was logged on.
	 */
	let { node }: { node: NodeResponse } = $props();

	let open = $state(false);
	let pokes = $state<PokeResponse[] | null>(null);

	async function show() {
		open = true;
		pokes = null;
		try {
			pokes = await nodesApi.pokes(node.id);
		} catch (error) {
			notifyError(error);
			open = false;
		}
	}

	function openPoked(nodeId: string) {
		open = false;
		openNode(nodeId);
	}
</script>

<!-- Keeps clicks (and the dialog's own) from reaching a clickable parent. -->
<!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
<span class="poke-history" onclick={(event) => event.stopPropagation()}>
	<button
		type="button"
		class="square"
		title="Poke history"
		aria-label="Poke history"
		onclick={show}
	>
		<Icon icon={ScrollText} always />
	</button>

	<Modal {open} onclose={() => (open = false)} label="Poke history" width={420}>
		<div class="dialog">
			<div class="label">Poke history</div>
			<div class="title">{node.title}</div>

			{#if pokes === null}
				<p class="empty">Loading…</p>
			{:else if pokes.length === 0}
				<p class="empty">No pokes yet.</p>
			{:else}
				<ol class="pokes">
					{#each pokes as poke (poke.id)}
						<li>
							<span class="when">{shortDate(poke.poked_at)} · {toTimeInput(poke.poked_at)}</span>
							{#if poke.node_id !== node.id}
								<button type="button" class="node" onclick={() => openPoked(poke.node_id)}>
									{graph.nodeById.get(poke.node_id)?.title ?? 'Unknown node'}
								</button>
							{/if}
						</li>
					{/each}
				</ol>
			{/if}
		</div>
	</Modal>
</span>

<style>
	.poke-history {
		display: inline-flex;
		align-self: stretch;
	}

	.square {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		align-self: stretch;
		aspect-ratio: 1;
		min-width: 28px;
		min-height: 28px;
		padding: 0;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		color: var(--accent);
		transition:
			opacity var(--fast) var(--ease),
			transform var(--fast) var(--ease);
	}

	.square:hover {
		opacity: 0.85;
	}

	.square:active {
		transform: translateY(1px);
	}

	.dialog {
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.title {
		margin-top: -8px;
		font: 700 20px/1.2 var(--font-display);
	}

	.empty {
		margin: 0;
		color: var(--ink-2);
		font: 500 13px/1.4 var(--font-display);
	}

	.pokes {
		list-style: none;
		margin: 0;
		padding: 0;
		max-height: 50vh;
		overflow-y: auto;
		border-top: var(--border-width-hair) solid var(--line);
	}

	.pokes li {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: 12px;
		padding: 8px 0;
		border-bottom: var(--border-width-hair) solid var(--line);
	}

	.when {
		font: 600 12px/1.3 var(--font-mono);
		white-space: nowrap;
	}

	.node {
		padding: 0;
		border: none;
		background: none;
		color: var(--accent);
		font: 600 13px/1.3 var(--font-display);
		text-align: right;
		cursor: pointer;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.node:hover {
		text-decoration: underline;
	}
</style>
