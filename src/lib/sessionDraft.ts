/** 会话输入草稿（未发送内容）的 localStorage key 前缀。 */
const KEY_PREFIX = "okey-session-draft:";

function draftKey(session_id: string) {
	return `${KEY_PREFIX}${session_id}`;
}

function readStored(session_id: string): string {
	try {
		return localStorage.getItem(draftKey(session_id)) ?? "";
	} catch {
		return "";
	}
}

/** 读取某会话已保存的草稿（无则空串）。 */
export function getSessionDraft(session_id: string): string {
	return readStored(session_id);
}

/** 保存草稿；空值会移除记录，避免残留 key。 */
export function setSessionDraft(session_id: string, value: string): void {
	try {
		if (value) localStorage.setItem(draftKey(session_id), value);
		else localStorage.removeItem(draftKey(session_id));
	} catch {
		// localStorage 不可用（隐私模式/配额超限）时静默降级为不持久化
	}
}

/** 新建会话时把 from 会话未发送的草稿携带到 to 会话。 */
export function carrySessionDraft(from: string, to: string): void {
	const draft = readStored(from);
	if (draft) setSessionDraft(to, draft);
}

/** 删除会话后清理其草稿。 */
export function clearSessionDraft(session_id: string): void {
	try {
		localStorage.removeItem(draftKey(session_id));
	} catch {
		// ignore
	}
}
