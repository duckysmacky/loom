<script lang="ts">
	import { Copy, Plus, Unplug } from '@lucide/svelte';
	import './settings.css';
	import Button from '$lib/components/ui/Button.svelte';
	import { mcpApi, oauthApi, tokensApi } from '$lib/api/endpoints';
	import { relativeDays, shortDate } from '$lib/graph/display';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import type { ConnectedClientResponse } from '$lib/types/ConnectedClientResponse';
	import type { McpInfoResponse } from '$lib/types/McpInfoResponse';
	import type { AccessTokenResponse } from '$lib/types/AccessTokenResponse';
	import type { TokenScope } from '$lib/types/TokenScope';

	let info = $state<McpInfoResponse | null>(null);
	let tokens = $state<AccessTokenResponse[]>([]);
	let clients = $state<ConnectedClientResponse[]>([]);
	let newName = $state('');
	let newScope = $state<TokenScope>('read');
	// Days until expiry; null = never.
	let newExpiry = $state<number | null>(90);
	let revealed = $state<string | null>(null);
	let revokingId = $state<string | null>(null);

	async function load() {
		try {
			[info, tokens] = await Promise.all([mcpApi.info(), tokensApi.list()]);
			// The OAuth routes only exist while the MCP server is on.
			if (info.enabled) clients = await oauthApi.clients();
		} catch (error) {
			notifyError(error);
		}
	}

	const apiUrl = `${window.location.origin}/api`;

	load();

	async function copy(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			notify('Copied');
		} catch {
			notify('Could not copy - select it and copy by hand', 'error');
		}
	}

	async function create(event: SubmitEvent) {
		event.preventDefault();
		const name = newName.trim();
		if (!name) return;
		try {
			const created = await tokensApi.create({ name, scope: newScope, expires_in_days: newExpiry });
			tokens = [created.token, ...tokens];
			revealed = created.raw_token;
			newName = '';
		} catch (error) {
			notifyError(error);
		}
	}

	const scopeLabel: Record<TokenScope, string> = { read: 'read-only', read_write: 'read & write' };

	function expiryText(token: AccessTokenResponse): string {
		if (!token.expires_at) return 'never expires';
		return new Date(token.expires_at) <= new Date()
			? 'expired'
			: `expires ${shortDate(token.expires_at)}`;
	}

	async function disconnect(client: ConnectedClientResponse) {
		revokingId = null;
		try {
			await oauthApi.revokeClient(client.client_id);
			clients = clients.filter((candidate) => candidate.client_id !== client.client_id);
			notify(`Disconnected “${client.client_name}”`);
		} catch (error) {
			notifyError(error);
		}
	}

	async function revoke(token: AccessTokenResponse) {
		revokingId = null;
		try {
			await tokensApi.remove(token.id);
			tokens = tokens.filter((candidate) => candidate.id !== token.id);
			notify(`Revoked “${token.name}”`);
		} catch (error) {
			notifyError(error);
		}
	}
</script>

<section class="settings-section">
	<h2>MCP server</h2>
	<p class="explain">
		Lets AI agents (Claude and other MCP clients) read and edit your graph. Anything connected has
		full access to your nodes.
	</p>
	{#if info === null}
		<p class="muted">Loading…</p>
	{:else if info.enabled && info.url}
		<div class="settings-row">
			<span>
				<span class="row-label">Server URL</span>
				<span class="row-help">
					Add it as a custom connector in Claude, or to any MCP client. Connectors sign in through
					Loom; clients that can send headers can use a token below instead.
				</span>
			</span>
			<span class="url-row">
				<code class="url">{info.url}</code>
				<Button variant="quiet" onclick={() => copy(info!.url!)} icon={Copy}>Copy</Button>
			</span>
		</div>
	{:else}
		<p class="muted">
			The MCP server is switched off on this instance. Set <code>MCP_ENABLED=true</code> and
			<code>PUBLIC_URL</code> in the server's <code>.env</code> to turn it on - see the README.
		</p>
	{/if}
</section>

{#if info?.enabled}
	<section class="settings-section">
		<h2>Connected apps</h2>
		<p class="explain">Apps you approved through Loom's sign-in page, like Claude connectors.</p>
		<ul class="tokens">
			{#each clients as client (client.client_id)}
				<li class="token">
					<span class="name">{client.client_name}</span>
					<span class="meta">
						connected {shortDate(client.connected_at)} · {client.last_used_at
							? `used ${relativeDays(client.last_used_at)}`
							: 'never used'}
					</span>
					{#if revokingId === client.client_id}
						<Button variant="quiet" onclick={() => (revokingId = null)}>Keep</Button>
						<Button variant="primary" onclick={() => disconnect(client)} icon={Unplug}
							>Disconnect</Button
						>
					{:else}
						<Button variant="quiet" onclick={() => (revokingId = client.client_id)} icon={Unplug}
							>Disconnect</Button
						>
					{/if}
				</li>
			{:else}
				<li class="empty">No connected apps.</li>
			{/each}
		</ul>
	</section>
{/if}

<section class="settings-section">
	<h2>Access tokens</h2>
	<p class="explain">
		For scripts, other websites and agents that send an <code>Authorization: Bearer</code> header -
		to the REST API at <code>{apiUrl}</code>{#if info?.enabled}
			or the MCP server{/if}. Read-only tokens can only read. A token is shown once - store it
		somewhere safe.
	</p>

	<form class="create" onsubmit={create}>
		<input
			class="field"
			placeholder="Token name, e.g. “laptop Claude Code”"
			aria-label="New token name"
			maxlength="100"
			bind:value={newName}
		/>
		<select class="field scope" aria-label="Token access" bind:value={newScope}>
			<option value="read">Read-only</option>
			<option value="read_write">Read & write</option>
		</select>
		<select class="field scope" aria-label="Token expiry" bind:value={newExpiry}>
			<option value={30}>Expires in 30 days</option>
			<option value={90}>Expires in 90 days</option>
			<option value={365}>Expires in 1 year</option>
			<option value={null}>Never expires</option>
		</select>
		<Button type="submit" variant="primary" disabled={!newName.trim()} icon={Plus}>Create</Button>
	</form>

	{#if revealed}
		<div class="revealed" role="status">
			<span class="row-help">Copy this token now - it won't be shown again.</span>
			<span class="url-row">
				<code class="url">{revealed}</code>
				<Button variant="quiet" onclick={() => copy(revealed!)} icon={Copy}>Copy</Button>
				<Button variant="quiet" onclick={() => (revealed = null)}>Done</Button>
			</span>
		</div>
	{/if}

	<ul class="tokens">
		{#each tokens as token (token.id)}
			<li class="token">
				<span class="name">{token.name}</span>
				<span class="meta">
					{scopeLabel[token.scope]} · {expiryText(token)} · created {shortDate(token.created_at)} · {token.last_used_at
						? `used ${relativeDays(token.last_used_at)}`
						: 'never used'}
				</span>
				{#if revokingId === token.id}
					<Button variant="quiet" onclick={() => (revokingId = null)}>Keep</Button>
					<Button variant="primary" onclick={() => revoke(token)} icon={Unplug}>Revoke</Button>
				{:else}
					<Button variant="quiet" onclick={() => (revokingId = token.id)} icon={Unplug}
						>Revoke</Button
					>
				{/if}
			</li>
		{:else}
			<li class="empty">No tokens yet.</li>
		{/each}
	</ul>
</section>

<style>
	.muted {
		margin: 0;
		color: var(--ink-2);
		font: 500 12.5px/1.5 var(--font-display);
	}

	.url-row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

	.url {
		font: 500 12px/1.4 var(--font-mono);
		padding: 6px 8px;
		border: var(--border-width-hair) solid var(--line);
		overflow-wrap: anywhere;
	}

	.create {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.create .field {
		flex: 1;
		min-width: 180px;
	}

	.create .scope {
		flex: 0 0 auto;
		min-width: 0;
	}

	.revealed {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 10px 12px;
		border: var(--border-width) solid var(--ink);
	}

	.tokens {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
	}

	.token {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
		padding: 10px 0;
		border-top: var(--border-width-hair) solid var(--line);
	}

	.name {
		font: 700 13.5px/1.2 var(--font-display);
		min-width: 120px;
	}

	.meta {
		font: 500 10.5px/1 var(--font-mono);
		color: var(--ink-2);
		margin-right: auto;
	}

	.empty {
		padding: 12px 0;
		color: var(--ink-2);
		border-top: var(--border-width-hair) solid var(--line);
	}
</style>
