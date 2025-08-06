Building a basic Google Docs alternative using Rust (backend) + React (frontend) with IP-based history tracking, versioning, and shared URL access, but without rich text formatting or authentication, is absolutely doable and can be kept relatively simple if you narrow the scope carefully.

Here’s a breakdown of what’s involved and how to approach it.

⸻

🔧 Functional Requirements (MVP)
	1.	Shared Document via URL
	•	URLs like /doc/abc123 open the document.
	•	Backend creates and stores documents with UUIDs or slugs.
	2.	Basic Real-Time Collaboration
	•	Users see edits from others in near real-time.
	•	Conflict resolution via last-write-wins (or CRDTs for better sync).
	3.	Edit History (IP-based)
	•	Log each change with timestamp and requestor’s IP.
	4.	Document Versioning
	•	Basic versioning: store snapshots of the document at intervals or per edit.

⸻

🧱 System Architecture

Backend (Rust)
	•	Framework: axum, actix-web, or warp
	•	Database: SQLite or PostgreSQL
	•	Key Endpoints:
	•	POST /doc – create a new document
	•	GET /doc/:id – fetch the current document
	•	PUT /doc/:id – apply edits
	•	GET /doc/:id/history – fetch history
	•	WebSocket: for real-time edit broadcasting (optional but ideal)
	•	Middleware: Capture IP address from request for logging

Frontend (React)
	•	Basic textarea or Markdown editor
	•	Auto-save on edit (debounced)
	•	Pull latest content from backend periodically or via WebSocket
	•	Display version number or change history

⸻

💡 Implementation Complexity

Feature	Complexity	Notes
Document creation & sharing	🟢 Easy	REST endpoint to generate new doc
IP tracking	🟢 Easy	Read from request headers or socket IP
Real-time editing	🟡 Medium	Easier with polling, better with WebSocket
Versioning	🟡 Medium	Per-edit save or periodic snapshots
Collaboration conflict handling	🟡🔴 Medium–Hard	CRDTs if you’re going fancy, otherwise just overwrite
No login	🟢 Easy	No auth layer keeps it simple


⸻

🕒 Estimated Effort (Solo Developer)

Task	Estimated Time
Backend CRUD (Rust)	1–2 days
Basic frontend editor (React)	1 day
IP-based logging + versioning	1–2 days
Real-time sync via polling	0.5–1 day
Optional: Real-time sync via WebSockets	1–2 days
UI polish / deployment	1–2 days

⏳ Total: ~5–9 days for a basic version

⸻

🧠 Tips
	•	Use uuid crate to generate unique document IDs.
	•	Use axum extractors to grab client IP.
	•	Store edits with:

document_id | timestamp | ip_address | content | version


	•	For versioning, you can:
	•	Store a full copy of the doc per version (simplest)
	•	Or, store diffs using libraries like diffy
	•	Consider using Monaco Editor in the frontend if you want better UX than textarea (optional).

⸻

🧪 Optional Enhancements
	•	Markdown preview pane
	•	Read-only mode
	•	Version rollback
	•	Display IPs next to edit events (like “anonymous wolf edited this line”)

⸻

✅ TL;DR

Yes, it’s very feasible. A basic Google Docs alternative using Rust + React can be done in under 2 weeks of part-time work. You don’t need CRDTs or OT unless you’re aiming for seamless, character-by-character collaborative editing. For MVP, REST + polling and basic version logging is plenty.

Want help scaffolding this out (folder structure, schema, etc.)?