import { queryString, request } from './client';
import { prefs } from '$lib/stores/prefs.svelte';
import type { ActivePeriodResponse } from '$lib/types/ActivePeriodResponse';
import type { AuthResponse } from '$lib/types/AuthResponse';
import type { AuthUserView } from '$lib/types/AuthUserView';
import type { CanvasResponse } from '$lib/types/CanvasResponse';
import type { ChecklistItemResponse } from '$lib/types/ChecklistItemResponse';
import type { ChangePasswordRequest } from '$lib/types/ChangePasswordRequest';
import type { CreateActivePeriodRequest } from '$lib/types/CreateActivePeriodRequest';
import type { CreatedMcpTokenResponse } from '$lib/types/CreatedMcpTokenResponse';
import type { CreateEdgeRequest } from '$lib/types/CreateEdgeRequest';
import type { CreateMcpTokenRequest } from '$lib/types/CreateMcpTokenRequest';
import type { CreateNodeRequest } from '$lib/types/CreateNodeRequest';
import type { CreateTopicRequest } from '$lib/types/CreateTopicRequest';
import type { DashboardResponse } from '$lib/types/DashboardResponse';
import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { LoginRequest } from '$lib/types/LoginRequest';
import type { McpInfoResponse } from '$lib/types/McpInfoResponse';
import type { McpTokenResponse } from '$lib/types/McpTokenResponse';
import type { NodeListQuery } from '$lib/types/NodeListQuery';
import type { NodeResponse } from '$lib/types/NodeResponse';
import type { PokeResponse } from '$lib/types/PokeResponse';
import type { RefreshResponse } from '$lib/types/RefreshResponse';
import type { ReorderNodesRequest } from '$lib/types/ReorderNodesRequest';
import type { SignupRequest } from '$lib/types/SignupRequest';
import type { TimelineResponse } from '$lib/types/TimelineResponse';
import type { TopicResponse } from '$lib/types/TopicResponse';
import type { UpdateActivePeriodRequest } from '$lib/types/UpdateActivePeriodRequest';
import type { UpdateChecklistItemRequest } from '$lib/types/UpdateChecklistItemRequest';
import type { UpdateNodeRequest } from '$lib/types/UpdateNodeRequest';
import type { UpdateTopicRequest } from '$lib/types/UpdateTopicRequest';

export const authApi = {
	signup: (body: SignupRequest) => request<AuthResponse>('POST', '/auth/signup', body),
	login: (body: LoginRequest) => request<AuthResponse>('POST', '/auth/login', body),
	logout: () => request<void>('POST', '/auth/logout'),
	me: () => request<AuthUserView>('GET', '/auth/me'),
	changePassword: (body: ChangePasswordRequest) =>
		request<RefreshResponse>('POST', '/auth/password', body)
};

export const nodesApi = {
	list: (query: NodeListQuery = {}) =>
		request<NodeResponse[]>(
			'GET',
			`/nodes${queryString({
				view: query.view,
				kind: query.kind,
				status: query.status,
				focus: query.focus
			})}`
		),
	get: (nodeId: string) => request<NodeResponse>('GET', `/nodes/${nodeId}`),
	create: (body: CreateNodeRequest) => request<NodeResponse>('POST', '/nodes', body),
	// `track_active_periods` is added here, not by the caller - it rides on
	// every PATCH (harmless when status isn't in the body; the backend only
	// acts on it for a real active-boundary transition), keeping the
	// "record active periods" setting a single frontend choke point instead
	// of something every status-changing call site has to remember to add.
	update: (nodeId: string, body: Omit<UpdateNodeRequest, 'track_active_periods'>) =>
		request<NodeResponse>('PATCH', `/nodes/${nodeId}`, {
			...body,
			track_active_periods: prefs.trackActivePeriods
		} satisfies UpdateNodeRequest),
	remove: (nodeId: string) => request<void>('DELETE', `/nodes/${nodeId}`),
	attachTopic: (nodeId: string, topicId: string) =>
		request<void>('POST', `/nodes/${nodeId}/topics`, { topic_id: topicId }),
	detachTopic: (nodeId: string, topicId: string) =>
		request<void>('DELETE', `/nodes/${nodeId}/topics/${topicId}`),
	poke: (nodeId: string) => request<PokeResponse>('POST', `/nodes/${nodeId}/pokes`),
	pokes: (nodeId: string) => request<PokeResponse[]>('GET', `/nodes/${nodeId}/pokes`),
	reorder: (nodeIds: string[]) =>
		request<void>('PUT', '/nodes/order', { node_ids: nodeIds } satisfies ReorderNodesRequest),
	clearOrder: () => request<void>('DELETE', '/nodes/order')
};

export const checklistApi = {
	list: (nodeId: string) => request<ChecklistItemResponse[]>('GET', `/nodes/${nodeId}/checklist`),
	add: (nodeId: string, title: string) =>
		request<ChecklistItemResponse>('POST', `/nodes/${nodeId}/checklist`, { title }),
	update: (itemId: string, body: UpdateChecklistItemRequest) =>
		request<ChecklistItemResponse>('PATCH', `/checklist/${itemId}`, body),
	remove: (itemId: string) => request<void>('DELETE', `/checklist/${itemId}`)
};

export const edgesApi = {
	create: (body: CreateEdgeRequest) => request<EdgeResponse>('POST', '/edges', body),
	remove: (edgeId: string) => request<void>('DELETE', `/edges/${edgeId}`)
};

export const topicsApi = {
	list: () => request<TopicResponse[]>('GET', '/topics'),
	create: (body: CreateTopicRequest) => request<TopicResponse>('POST', '/topics', body),
	update: (topicId: string, body: UpdateTopicRequest) =>
		request<TopicResponse>('PATCH', `/topics/${topicId}`, body),
	remove: (topicId: string) => request<void>('DELETE', `/topics/${topicId}`)
};

export const periodsApi = {
	list: (nodeId: string) => request<ActivePeriodResponse[]>('GET', `/nodes/${nodeId}/periods`),
	add: (nodeId: string, body: CreateActivePeriodRequest) =>
		request<ActivePeriodResponse>('POST', `/nodes/${nodeId}/periods`, body),
	update: (periodId: string, body: UpdateActivePeriodRequest) =>
		request<ActivePeriodResponse>('PATCH', `/periods/${periodId}`, body),
	remove: (periodId: string) => request<void>('DELETE', `/periods/${periodId}`)
};

export const boardApi = {
	canvas: () => request<CanvasResponse>('GET', '/board/canvas'),
	timeline: () => request<TimelineResponse>('GET', '/board/timeline')
};

export const mcpApi = {
	info: () => request<McpInfoResponse>('GET', '/mcp/info'),
	tokens: () => request<McpTokenResponse[]>('GET', '/mcp/tokens'),
	createToken: (body: CreateMcpTokenRequest) =>
		request<CreatedMcpTokenResponse>('POST', '/mcp/tokens', body),
	removeToken: (tokenId: string) => request<void>('DELETE', `/mcp/tokens/${tokenId}`)
};

export const dashboardApi = {
	get: () => request<DashboardResponse>('GET', '/dashboard')
};
