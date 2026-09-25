import { queryString, request } from './client';
import type { AuthResponse } from '$lib/types/AuthResponse';
import type { AuthUserView } from '$lib/types/AuthUserView';
import type { CanvasResponse } from '$lib/types/CanvasResponse';
import type { ChecklistItemResponse } from '$lib/types/ChecklistItemResponse';
import type { ChangePasswordRequest } from '$lib/types/ChangePasswordRequest';
import type { CreateEdgeRequest } from '$lib/types/CreateEdgeRequest';
import type { CreateNodeRequest } from '$lib/types/CreateNodeRequest';
import type { CreateTopicRequest } from '$lib/types/CreateTopicRequest';
import type { DashboardResponse } from '$lib/types/DashboardResponse';
import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { LoginRequest } from '$lib/types/LoginRequest';
import type { NodeListQuery } from '$lib/types/NodeListQuery';
import type { NodeResponse } from '$lib/types/NodeResponse';
import type { PokeResponse } from '$lib/types/PokeResponse';
import type { RefreshResponse } from '$lib/types/RefreshResponse';
import type { ReorderNodesRequest } from '$lib/types/ReorderNodesRequest';
import type { SignupRequest } from '$lib/types/SignupRequest';
import type { TopicResponse } from '$lib/types/TopicResponse';
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
	update: (nodeId: string, body: UpdateNodeRequest) =>
		request<NodeResponse>('PATCH', `/nodes/${nodeId}`, body),
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

export const boardApi = {
	canvas: () => request<CanvasResponse>('GET', '/board/canvas')
};

export const dashboardApi = {
	get: () => request<DashboardResponse>('GET', '/dashboard')
};
