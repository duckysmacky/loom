<script lang="ts">
	import { m } from '$lib/paraglide/messages';
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
			notify(m.conn_copied());
		} catch {
			notify(m.conn_copy_failed(), 'error');
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

	const scopeLabel: Record<TokenScope, () => string> = {
		read: m.conn_scope_read,
		read_write: m.conn_scope_read_write
	};

	function expiryText(token: AccessTokenResponse): string {
		if (!token.expires_at) return m.conn_never_expires();
		return new Date(token.expires_at) <= new Date()
			? m.conn_expired()
			: m.conn_expires({ date: shortDate(token.expires_at) });
	}

	async function disconnect(client: ConnectedClientResponse) {
		revokingId = null;
		try {
			await oauthApi.revokeClient(client.client_id);
			clients = clients.filter((candidate) => candidate.client_id !== client.client_id);
			notify(m.conn_disconnected({ name: client.client_name }));
		} catch (error) {
			notifyError(error);
		}
	}

	async function revoke(token: AccessTokenResponse) {
		revokingId = null;
		try {
			await tokensApi.remove(token.id);
			tokens = tokens.filter((candidate) => candidate.id !== token.id);
			notify(m.conn_revoked({ name: token.name }));
		} catch (error) {
			notifyError(error);
		}
	}
</script>

<section class="settings-section">
	<h2>{m.conn_mcp_server()}</h2>
	<p class="explain">
		{m.conn_lets_ai_agents_claude()}
	</p>
	{#if info === null}
		<p class="muted">{m.loading()}</p>
	{:else if info.enabled && info.url}
		<div class="settings-row">
			<span>
				<span class="row-label">{m.conn_server_url()}</span>
				<span class="row-help">
					{m.conn_add_as_custom_connector()}
				</span>
			</span>
			<span class="url-row">
				<code class="url">{info.url}</code>
				<Button variant="quiet" onclick={() => copy(info!.url!)} icon={Copy}>{m.conn_copy()}</Button
				>
			</span>
		</div>
	{:else}
		<p class="muted">
			{m.conn_mcp_server_switched_off()} <code>MCP_ENABLED=true</code>
			{m.conn_and()}
			<code>PUBLIC_URL</code>
			{m.conn_in_server()} <code>.env</code>
			{m.conn_turn_see_readme()}
		</p>
	{/if}
</section>

{#if info?.enabled}
	<section class="settings-section">
		<h2>{m.conn_connected_apps()}</h2>
		<p class="explain">{m.conn_apps_you_approved_through()}</p>
		<ul class="tokens">
			{#each clients as client (client.client_id)}
				<li class="token">
					<span class="name">{client.client_name}</span>
					<span class="meta">
						{m.conn_connected_on({ date: shortDate(client.connected_at) })} · {client.last_used_at
							? m.conn_used({ when: relativeDays(client.last_used_at) })
							: m.conn_never_used()}
					</span>
					{#if revokingId === client.client_id}
						<Button variant="quiet" onclick={() => (revokingId = null)}>{m.action_keep()}</Button>
						<Button variant="primary" onclick={() => disconnect(client)} icon={Unplug}
							>{m.conn_disconnect()}</Button
						>
					{:else}
						<Button variant="quiet" onclick={() => (revokingId = client.client_id)} icon={Unplug}
							>{m.conn_disconnect()}</Button
						>
					{/if}
				</li>
			{:else}
				<li class="empty">{m.conn_no_connected_apps()}</li>
			{/each}
		</ul>
	</section>
{/if}

<section class="settings-section">
	<h2>{m.conn_access_tokens()}</h2>
	<p class="explain">
		{m.conn_scripts_other_websites_agents()} <code>Authorization: Bearer</code>
		{m.conn_header_rest_api_at()} <code>{apiUrl}</code>{#if info?.enabled}
			{m.conn_or_mcp()}{/if}. {m.conn_tokens_explain_end()}
	</p>

	<form class="create" onsubmit={create}>
		<input
			class="field"
			placeholder={m.conn_token_name_e_g()}
			aria-label={m.conn_new_token_name()}
			maxlength="100"
			bind:value={newName}
		/>
		<select class="field scope" aria-label={m.conn_token_access()} bind:value={newScope}>
			<option value="read">{m.conn_read_only()}</option>
			<option value="read_write">{m.conn_read_write()}</option>
		</select>
		<select class="field scope" aria-label={m.conn_token_expiry()} bind:value={newExpiry}>
			<option value={30}>{m.conn_expires_30_days()}</option>
			<option value={90}>{m.conn_expires_90_days()}</option>
			<option value={365}>{m.conn_expires_1_year()}</option>
			<option value={null}>{m.conn_expiry_never()}</option>
		</select>
		<Button type="submit" variant="primary" disabled={!newName.trim()} icon={Plus}
			>{m.action_create()}</Button
		>
	</form>

	{#if revealed}
		<div class="revealed" role="status">
			<span class="row-help">{m.conn_copy_this_token_now()}</span>
			<span class="url-row">
				<code class="url">{revealed}</code>
				<Button variant="quiet" onclick={() => copy(revealed!)} icon={Copy}>{m.conn_copy()}</Button>
				<Button variant="quiet" onclick={() => (revealed = null)}>{m.action_done()}</Button>
			</span>
		</div>
	{/if}

	<ul class="tokens">
		{#each tokens as token (token.id)}
			<li class="token">
				<span class="name">{token.name}</span>
				<span class="meta">
					{scopeLabel[token.scope]()} · {expiryText(token)} · {m.conn_created_on({
						date: shortDate(token.created_at)
					})} · {token.last_used_at
						? m.conn_used({ when: relativeDays(token.last_used_at) })
						: m.conn_never_used()}
				</span>
				{#if revokingId === token.id}
					<Button variant="quiet" onclick={() => (revokingId = null)}>{m.action_keep()}</Button>
					<Button variant="primary" onclick={() => revoke(token)} icon={Unplug}
						>{m.conn_revoke()}</Button
					>
				{:else}
					<Button variant="quiet" onclick={() => (revokingId = token.id)} icon={Unplug}
						>{m.conn_revoke()}</Button
					>
				{/if}
			</li>
		{:else}
			<li class="empty">{m.conn_no_tokens_yet()}</li>
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
