<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Tabs from '$lib/components/ui/Tabs.svelte';
	import { withParam } from '$lib/navigation';
	import AccountTab from './AccountTab.svelte';
	import AppearanceTab from './AppearanceTab.svelte';
	import ConnectionsTab from './ConnectionsTab.svelte';
	import FunctionalityTab from './FunctionalityTab.svelte';
	import TopicsTab from './TopicsTab.svelte';

	const TABS = [
		{ value: 'account', label: 'Account' },
		{ value: 'appearance', label: 'Appearance' },
		{ value: 'topics', label: 'Topics & tags' },
		{ value: 'functionality', label: 'Functionality' },
		{ value: 'connections', label: 'Connections' }
	] as const;
	type Tab = (typeof TABS)[number]['value'];

	const tab = $derived<Tab>(
		TABS.find((candidate) => candidate.value === page.url.searchParams.get('tab'))?.value ??
			'account'
	);
</script>

<Tabs
	tabs={[...TABS]}
	value={tab}
	onchange={(next) => goto(withParam('tab', next), { noScroll: true, keepFocus: true })}
/>

<div class="page">
	{#if tab === 'account'}
		<AccountTab />
	{:else if tab === 'appearance'}
		<AppearanceTab />
	{:else if tab === 'topics'}
		<TopicsTab />
	{:else if tab === 'functionality'}
		<FunctionalityTab />
	{:else}
		<ConnectionsTab />
	{/if}
</div>

<style>
	.page {
		padding: var(--page-pad);
		max-width: 760px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
</style>
