// Pure client-side SPA: adapter-static emits one index.html fallback and the
// Axum backend serves it for every non-API path. Nothing is prerendered - every
// page depends on the signed-in user's data.
export const ssr = false;
