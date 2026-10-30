<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { ArrowUpRight } from '@lucide/svelte';
	import { KIND_ICON } from '$lib/components/KindMark.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify } from '$lib/stores/toasts.svelte';
	import { kindChangeLosses } from '$lib/graph/display';
	import { kindLabel } from '$lib/i18n';
	import type { NodeFocus } from '$lib/types/NodeFocus';
	import type { NodeKind } from '$lib/types/NodeKind';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	/**
	 * Promotes an idea in place - same row, same id, same
	 * edges - by giving it a real kind, a status (queued/active) and a focus
	 * tier in one PATCH. Ideas have none of those until promoted.
	 */
	let { node, onclose }: { node: NodeResponse | null; onclose: () => void } = $props();

	let kind = $state<NodeKind>('project');
	let status = $state<'queued' | 'active'>('queued');
	let focus = $state<NodeFocus>('secondary');
	let saving = $state(false);

	// Reset each time the dialog opens.
	$effect(() => {
		if (!node) return;
		kind = 'project';
		focus = node.focus ?? 'secondary';
		status = 'queued';
	});

	const losses = $derived(node ? kindChangeLosses(node, kind) : []);

	async function promote() {
		if (!node) return;
		saving = true;
		const updated = await graph.mutate(() =>
			nodesApi.update(node!.id, { kind, focus, ...(kind === 'path' ? {} : { status }) })
		);
		saving = false;
		if (updated) {
			notify(m.promoted({ title: updated.title, kind: kindLabel(kind).toLocaleLowerCase() }));
			onclose();
		}
	}
</script>

<Modal open={node !== null} {onclose} label={m.promote_idea()} width={460}>
	<div class="dialog">
		<div class="label">{m.promote_idea()}</div>
		<div class="title">{node?.title}</div>

		<div class="field-group">
			<span class="label">{m.field_kind()}</span>
			<SegmentedControl
				label={m.field_kind()}
				value={kind}
				onchange={(value) => (kind = value)}
				options={[
					{ value: 'project', label: m.kind_project(), icon: KIND_ICON.project },
					{ value: 'study', label: m.kind_study(), icon: KIND_ICON.study },
					{ value: 'path', label: m.kind_path(), icon: KIND_ICON.path }
				]}
			/>
		</div>
		{#if losses.length}
			<p class="losses">{m.promote_losses({ losses: losses.join(m.list_and()) })}</p>
		{/if}
		<!-- A path's status is derived from what's inside it. -->
		{#if kind !== 'path'}
			<div class="field-group">
				<span class="label">{m.field_status()}</span>
				<SegmentedControl
					label={m.field_status()}
					value={status}
					onchange={(value) => (status = value)}
					options={[
						{ value: 'queued', label: m.status_queued() },
						{ value: 'active', label: m.status_active() }
					]}
				/>
			</div>
		{/if}
		<div class="field-group">
			<span class="label">{m.field_focus_tier()}</span>
			<SegmentedControl
				label={m.field_focus_tier()}
				value={focus}
				onchange={(value) => (focus = value)}
				options={[
					{ value: 'primary', label: m.focus_primary() },
					{ value: 'secondary', label: m.focus_secondary() },
					{ value: 'background', label: m.focus_background() }
				]}
			/>
		</div>

		<div class="actions">
			<Button variant="quiet" onclick={onclose}>{m.action_cancel()}</Button>
			<Button variant="primary" onclick={promote} disabled={saving} icon={ArrowUpRight}
				>{m.promote_submit()}</Button
			>
		</div>
	</div>
</Modal>

<style>
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

	.field-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
		align-items: flex-start;
	}

	.losses {
		margin: -6px 0 0;
		font: 600 12px/1.4 var(--font-display);
		color: var(--warn);
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding-top: 14px;
		border-top: var(--border-width-hair) solid var(--line);
	}
</style>
