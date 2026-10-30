<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import { withParam } from '$lib/navigation';
	import { syncState } from '$lib/stores/prefs.svelte';
	import AccountTab from './AccountTab.svelte';
	import AppearanceTab from './AppearanceTab.svelte';
	import ConnectionsTab from './ConnectionsTab.svelte';
	import FeaturesTab from './FeaturesTab.svelte';

	import TopicsTab from './TopicsTab.svelte';

	const TABS = [
		{ value: 'account', label: 'Account' },
		{ value: 'appearance', label: 'Appearance' },
		{ value: 'topics', label: 'Topics & tags' },
		{ value: 'features', label: 'Features' },
		{ value: 'connections', label: 'Connections' }
	] as const;
	type Tab = (typeof TABS)[number]['value'];

	// `functionality` is the old name of the features tab (kept for old links).
	const requestedTab = $derived(
		page.url.searchParams.get('tab') === 'functionality'
			? 'features'
			: page.url.searchParams.get('tab')
	);
	const tab = $derived<Tab>(
		TABS.find((candidate) => candidate.value === requestedTab)?.value ?? 'account'
	);
</script>

<Tabs
	tabs={[...TABS]}
	value={tab}
	onchange={(next) => goto(withParam('tab', next), { noScroll: true, keepFocus: true })}
/>

<div class="page">
	{#if tab === 'appearance' || tab === 'features'}
		<p class="sync" class:failed={syncState.status === 'error'} role="status">
			{#if syncState.status === 'saving'}
				Saving…
			{:else if syncState.status === 'error'}
				Couldn't save to your account. Kept on this device; retries on your next change.
			{:else}
				Changes save to your account automatically.
			{/if}
		</p>
	{/if}
	{#if tab === 'account'}
		<AccountTab />
	{:else if tab === 'appearance'}
		<AppearanceTab />
	{:else if tab === 'topics'}
		<TopicsTab />
	{:else if tab === 'features'}
		<FeaturesTab />
	{:else}
		<ConnectionsTab />
	{/if}
</div>

<style>
	.sync {
		margin: 0;
		font: 500 12px/1.4 var(--font-display);
		color: var(--ink-2);
	}

	.sync.failed {
		color: var(--warn);
	}

	.page {
		padding: var(--page-pad);
		max-width: 760px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
</style>
