# MEW Storage Pattern

**Version:** 1.0
**Status:** Draft
**Scope:** Content storage, external data, and the control/data plane separation

---

# Part I: Motivation

## 1.1 The Core Problem

MEW graphs model entities and relationships. Some entities have associated content — documents contain text, images contain pixels, videos contain frames. This content can be arbitrarily large: kilobytes, megabytes, gigabytes.

Storing large binary content as node attributes creates fundamental problems:

| Problem | Consequence |
|---------|-------------|
| **Columnar storage breaks** | Variable-length blobs destroy memory locality |
| **Replication explodes** | Every observer copies every byte |
| **Queries slow down** | Scanning includes irrelevant bulk data |
| **Memory pressure** | Large attributes don't fit in working memory |
| **GPU unfriendly** | Variable-length data can't be tensorized |

The naive solution — storing everything as attributes — doesn't scale.

## 1.2 The Core Insight

**Separate control plane from data plane.**

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CONTROL PLANE vs DATA PLANE                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   CONTROL PLANE (Graph)                 DATA PLANE (Storage)        │
│   ─────────────────────                 ────────────────────        │
│                                                                      │
│   Structure                             Content                     │
│   Relationships                         Bytes                       │
│   Metadata                              Blobs                       │
│   Hashes (identifiers)                  Streams                     │
│                                                                      │
│   "What exists and how                  "What it contains"          │
│    things relate"                                                   │
│                                                                      │
│   Small, fixed-size                     Large, variable             │
│   Queryable, filterable                 Retrieved, streamed         │
│   Replicated everywhere                 Stored selectively          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

The graph stores identity and metadata. External storage holds bytes. The connection is a hash — a fixed-size identifier that uniquely names content without containing it.

## 1.3 The Physical Analogy

A library catalog card contains:
- Title, author, ISBN (metadata)
- Shelf location (reference to physical item)
- The card does NOT contain the book's pages

The book itself sits on the shelf. The catalog enables finding it.

MEW works the same way:
- Content node holds metadata (size, type, hash)
- Hash references bytes in external storage
- Graph enables finding, relating, authorizing
- Storage enables retrieving actual content

## 1.4 Design Principles

| Principle | Meaning |
|-----------|---------|
| **Kernel never touches bytes** | Graph stores hashes, never fetches content |
| **Content-addressed** | Hash of bytes IS the identifier |
| **Storage is client concern** | Clients resolve hashes to bytes |
| **Encryption is client concern** | Clients encrypt/decrypt; graph stores opaque wrapped keys |
| **Delegation is explicit** | Trust anchors are visible graph structure |
| **Chunking is transparent** | Large content split into chunks; graph doesn't know |

---

# Part II: Content Model

## 2.1 The `Hash` Scalar Type

`Hash` is a scalar type alongside `String`, `Int`, `Float`, `Bool`, `Timestamp`:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         HASH TYPE                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Properties:                                                       │
│   • Fixed size (32 bytes for SHA-256, or with algorithm prefix)    │
│   • Comparable, hashable, indexable                                │
│   • Displayed as hex or base58                                     │
│   • Stored inline in columnar storage                              │
│                                                                      │
│   Generic: usable for content hashes, merkle roots, integrity      │
│   checks, deduplication keys, or any cryptographic identifier.     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 Content-Addressing

Content is identified by the hash of its bytes:

```
hash = sha256(bytes)
```

This provides:

| Property | Benefit |
|----------|---------|
| **Deterministic** | Same bytes → same hash, always |
| **Integrity** | Can verify bytes match hash |
| **Immutable** | Changing bytes = new hash = new identity |
| **Deduplication** | Same content stored once, regardless of how many nodes reference it |

## 2.3 The Content Node Pattern

A standard pattern for content nodes (convention, not enforced by kernel):

```
node Content {
  hash: Hash [required, unique]
  size: Int [required]
  mime_type: String?
  created_at: Timestamp [required] = now()
}
```

This is a regular node. The kernel stores the `hash` attribute like any other. Clients know by convention that `Content.hash` references bytes in external storage.

## 2.4 Extending Content Types

Users can define specialized content types:

```
node Content {
  hash: Hash [required, unique]
  size: Int [required]
  mime_type: String?
  created_at: Timestamp [required] = now()
}

node Document : Content {
  title: String?
  page_count: Int?
  language: String?
}

node Video : Content {
  duration_ms: Int?
  width: Int?
  height: Int?
  codec: String?
}
```

Or define entirely separate content types:

```
node Attachment {
  hash: Hash [required, unique]
  size: Int [required]
  filename: String [required]
  uploaded_at: Timestamp [required] = now()
}
```

## 2.6 Relating Content to Entities

Content connects to entities via edges:

```
node Document {
  title: String [required]
  author: String
  status: String
}

edge has_content(entity: any, content: Content)

-- Usage
SPAWN doc: Document { title = "Report", author = "Alice" }
SPAWN c: Content { hash = #abc123, size = 1048576, mime_type = "application/pdf" }
LINK has_content(doc, c)
```

## 2.8 Versioning

Content is immutable (hash-addressed). Versioning is edges:

```
edge has_version(entity: any, content: Content) {
  version: Int [required]
  created_at: Timestamp [required] = now()
  created_by: any?
}

-- Or explicit current pointer
edge current_content(entity: any, content: Content)
```

Editing creates new content:

```
Document#123
  └──[has_version v=1]──▶ Content (hash: "abc...")
  └──[has_version v=2]──▶ Content (hash: "def...")
  └──[has_version v=3]──▶ Content (hash: "ghi...")  ← latest
  └──[current_content]───▶ Content (hash: "ghi...")
```

## 2.9 Deduplication

Same bytes = same hash = can be same Content node:

```
Document A ──[has_content]──▶ Content (hash: "abc")
Document B ──[has_content]──▶ Content (hash: "abc")  ← same node
```

Benefits:
- Storage: bytes stored once
- GC: reference counting via edges
- Integrity: hash equality = content equality

---

# Part III: Storage Architecture

## 3.1 The Kernel Boundary

The kernel never touches content bytes:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    KERNEL BOUNDARY                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   KERNEL                                                            │
│   ──────                                                            │
│   • Stores Hash as opaque 32-byte value                            │
│   • Indexes, queries, replicates hashes like any attribute         │
│   • Knows nothing about storage backends                           │
│   • Never fetches, stores, or validates bytes                      │
│   • No special semantics for Hash — just another scalar type       │
│                                                                      │
│   CLIENT                                                            │
│   ──────                                                            │
│   • Reads storage configuration                                    │
│   • Connects to storage backend                                    │
│   • Uploads bytes, gets hash                                       │
│   • Downloads bytes by hash                                        │
│   • Handles chunking, encryption, streaming                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.2 Storage Backend Pattern

Storage configuration is graph structure. A `StorageBackend` node holds the info clients need to connect:

```
node StorageBackend {
  name: String [required, unique]
  backend_type: String [required]
  endpoint: String?
  bucket: String?
  region: String?
  prefix: String?
  config_json: String?
}
```

Content links to its storage backend:

```
edge stored_in(content: Content, backend: StorageBackend)
```

## 3.3 Storage Interface

Clients implement a content-addressed storage interface:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CONTENT STORE INTERFACE                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Operations:                                                       │
│                                                                      │
│   put(bytes) → Hash                                                │
│     Store content, return hash                                     │
│                                                                      │
│   get(hash) → bytes | null                                         │
│     Retrieve content by hash                                       │
│                                                                      │
│   exists(hash) → bool                                              │
│     Check if content exists                                        │
│                                                                      │
│   delete(hash) → bool                                              │
│     Remove content (for GC)                                        │
│                                                                      │
│   list() → [Hash]                                                  │
│     Enumerate all stored hashes (for GC)                           │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

Implementations exist for various backends: local filesystem, S3, IPFS, Azure Blob, GCS, etc.

## 3.4 Chunking

Large content is split into fixed-size chunks. This is a storage-layer concern, invisible to the graph.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CHUNKED STORAGE                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Content bytes split into chunks (e.g., 1MB each).                │
│   Each chunk stored separately by its hash.                        │
│   Manifest lists chunks in order.                                  │
│   Content hash = hash of manifest.                                 │
│                                                                      │
│   Manifest structure:                                               │
│   {                                                                 │
│     "total_size": 10485760,                                        │
│     "chunk_size": 1048576,                                         │
│     "chunks": [                                                    │
│       { "hash": "abc...", "size": 1048576 },                      │
│       { "hash": "def...", "size": 1048576 },                      │
│       ...                                                          │
│     ]                                                              │
│   }                                                                  │
│                                                                      │
│   Benefits:                                                         │
│   • Streaming: read/write without buffering entire content        │
│   • Parallel: upload/download multiple chunks simultaneously      │
│   • Random access: seek to any offset, fetch relevant chunk       │
│   • Resume: track progress per chunk                              │
│   • Deduplication: identical chunks share storage                 │
│                                                                      │
│   Graph stores only the manifest hash. Chunking is invisible.      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 3.5 Garbage Collection

When Content nodes are killed, bytes become orphaned. A GC Agent cleans up:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    GC AGENT PATTERN                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   1. Query graph: all live Content hashes                          │
│      MATCH c: Content RETURN c.hash                                │
│                                                                      │
│   2. Query storage: all stored hashes                              │
│      storage.list()                                                │
│                                                                      │
│   3. Compute orphans:                                               │
│      orphans = stored_hashes - live_hashes                         │
│                                                                      │
│   4. Delete orphans:                                                │
│      for hash in orphans: storage.delete(hash)                     │
│                                                                      │
│   Properties:                                                       │
│   • Decoupled: kernel knows nothing about GC                       │
│   • Eventual: orphans exist temporarily                            │
│   • Safe: only deletes unreferenced content                        │
│   • Configurable: retention periods, policies                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part IV: Encryption Model

## 4.1 The Challenge

Content may be private. Encryption protects bytes. But how does sharing work?

```
Alice creates encrypted content.
Graph authorizes Bob to access.
Bob needs to decrypt.
Alice may be offline.
```

The key must flow from Alice to Bob without requiring Alice's presence.

## 4.2 The Solution: Explicit Delegation

Trust anchors are explicit graph structure. Alice delegates key authority to an agent who can distribute keys on her behalf.

```
┌─────────────────────────────────────────────────────────────────────┐
│                    DELEGATION MODEL                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   content_key edge                                                  │
│   ────────────────                                                  │
│   "Holder can decrypt this content"                                │
│                                                                      │
│   edge content_key(content: Content, holder: any) {                │
│     wrapped_key: Bytes [required]                                  │
│     granted_at: Timestamp [required] = now()                       │
│     granted_by: any?                                               │
│   }                                                                  │
│                                                                      │
│   wrapped_key = encrypt(DEK, holder.public_key)                    │
│   Only holder's private key can unwrap.                            │
│                                                                      │
│                                                                      │
│   key_authority edge                                                │
│   ──────────────────                                                │
│   "Authority can grant decryption to others"                       │
│                                                                      │
│   edge key_authority(content: Content, authority: any) {           │
│     wrapped_key: Bytes [required]                                  │
│     delegated_at: Timestamp [required] = now()                     │
│     delegated_by: any?                                             │
│   }                                                                  │
│                                                                      │
│   Authority holds DEK, can wrap it for new recipients.             │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.3 Encryption Flow

**Alice creates encrypted content:**

```
1. Generate DEK (data encryption key)
2. Encrypt plaintext with DEK → ciphertext
3. Upload ciphertext to storage → hash
4. SPAWN Content node with hash
5. LINK content_key(content, alice) with DEK wrapped for alice
6. LINK key_authority(content, key_agent) with DEK wrapped for agent
```

Alice explicitly delegates to key_agent. This is visible in the graph.

**Bob gets authorized:**

```
1. Authorization granted: LINK can_read(bob, content)

2. Key Agent (watching via subscription) sees new authorization

3. Key Agent distributes key:
   - Unwrap DEK using agent's private key
   - Wrap DEK for Bob's public key
   - LINK content_key(content, bob) with wrapped key

4. Bob reads content:
   - Query content_key(content, bob) → wrapped_key
   - Unwrap with bob's private key → DEK
   - Fetch ciphertext from storage
   - Decrypt with DEK → plaintext
```

Alice was never involved. Key Agent acted on her behalf.

## 4.4 Why This Works

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TRUST MODEL                                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Kernel sees:                                                      │
│   • content_key edges with opaque wrapped_key bytes                │
│   • key_authority edges with opaque wrapped_key bytes              │
│                                                                      │
│   Kernel cannot:                                                    │
│   • Decrypt wrapped_key (no private keys)                          │
│   • Interpret these as "special" (they're just edges)              │
│   • Access plaintext content                                       │
│                                                                      │
│   Trust is explicit:                                                │
│   • Alice chose to delegate to key_agent                           │
│   • This choice is visible: key_authority edge exists              │
│   • Auditable: query who has authority over what                   │
│                                                                      │
│   Flexibility:                                                      │
│   • No delegation: only Alice can share (requires online)          │
│   • Single agent: convenient, trust one entity                     │
│   • Multiple agents: redundancy, threshold schemes                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4.5 Encryption is Optional

These edges are only defined if encryption is needed. Unencrypted content simply has no `content_key` or `key_authority` edges:

```
-- Unencrypted content: just the Content node
SPAWN c: Content { hash = #abc, size = 1024 }
LINK has_content(doc, c)
-- Anyone who can query the Content node can fetch bytes

-- Encrypted content: add key edges
SPAWN c: Content { hash = #xyz, size = 1024 }
LINK has_content(doc, c)
LINK content_key(c, alice) { wrapped_key = 0x... }
LINK key_authority(c, agent) { wrapped_key = 0x... }
-- Only those with content_key edges can decrypt
```

The schema decides whether encryption types exist. The application decides whether to use them.

## 4.6 Chunked Encryption

For large content, each chunk is encrypted independently:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CHUNKED ENCRYPTION                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Each chunk encrypted with DEK + unique nonce.                    │
│   Chunks independently decryptable.                                │
│   Enables streaming decryption.                                    │
│                                                                      │
│   chunk_0 = encrypt(plaintext[0:1MB], DEK, nonce=0)               │
│   chunk_1 = encrypt(plaintext[1MB:2MB], DEK, nonce=1)             │
│   ...                                                               │
│                                                                      │
│   Read flow:                                                        │
│   1. Fetch and decrypt content_key → DEK                          │
│   2. Fetch manifest                                                │
│   3. For each chunk (or range):                                    │
│      a. Fetch chunk                                                │
│      b. Decrypt with DEK + chunk nonce                            │
│      c. Stream plaintext                                           │
│                                                                      │
│   Memory usage: O(chunk_size), not O(file_size)                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part V: Client Workflows

## 5.1 Write Content (Unencrypted)

```
1. Read source bytes
2. Get storage from World configuration
3. Upload bytes → hash
4. SPAWN Content { hash, size, mime_type }
5. LINK to owning entity
```

## 5.2 Write Content (Encrypted)

```
1. Read source bytes
2. Generate DEK
3. Encrypt bytes with DEK → ciphertext
4. Upload ciphertext → hash
5. SPAWN Content { hash, size (of plaintext), mime_type }
6. LINK to owning entity
7. LINK content_key to self with wrapped DEK
8. (Optional) LINK key_authority to agent with wrapped DEK
```

## 5.3 Read Content (Unencrypted)

```
1. MATCH Content → hash
2. Get storage from World configuration
3. Download bytes by hash
```

## 5.4 Read Content (Encrypted)

```
1. MATCH Content and content_key edge → hash, wrapped_key
2. Unwrap DEK using private key
3. Get storage from World configuration
4. Download ciphertext by hash
5. Decrypt with DEK → plaintext
```

## 5.5 Grant Access (With Delegation)

```
1. LINK authorization edge (can_read, etc.)
2. Key Agent observes authorization change
3. Key Agent fetches own wrapped DEK from key_authority
4. Key Agent unwraps DEK
5. Key Agent wraps DEK for new recipient
6. Key Agent creates content_key edge for recipient
```

## 5.6 Revoke Access

```
1. UNLINK authorization edge
2. Key Agent observes revocation
3. Key Agent removes content_key edge for revoked user
4. User can no longer query for wrapped key

Note: If user cached DEK, they can still decrypt.
True revocation requires re-encryption with new DEK.
```

---

# Part VI: Agent Patterns

## 6.1 GC Agent

Cleans orphaned content from storage:

```
Subscribe to Content deletions
Periodically:
  live_hashes = MATCH c: Content RETURN c.hash
  stored_hashes = storage.list()
  orphans = stored_hashes - live_hashes
  for hash in orphans (with retention policy):
    storage.delete(hash)
```

## 6.2 Key Distribution Agent

Distributes encryption keys when authorization changes:

```
Subscribe to authorization edges (can_read, etc.)

On new authorization(actor, content):
  If key_authority(content, self) exists:
    wrapped_dek = key_authority.wrapped_key
    dek = unwrap(wrapped_dek, self.private_key)
    recipient_wrapped = wrap(dek, actor.public_key)
    LINK content_key(content, actor) { wrapped_key = recipient_wrapped }

On revoked authorization(actor, content):
  UNLINK content_key(content, actor)
```

## 6.3 Search Agent

Indexes content for full-text search:

```
Subscribe to new Content nodes

On new Content:
  bytes = storage.get(content.hash)
  text = extract_text(bytes, content.mime_type)
  terms = tokenize(text)
  for term in terms:
    LINK contains_term(content, term)
```

Search becomes graph traversal:

```
MATCH t: Term, contains_term(c, t)
WHERE t.value = "MEW"
RETURN c
```

## 6.4 Transform Agent

Creates derived content (thumbnails, transcodes, etc.):

```
Subscribe to new ImageContent nodes

On new ImageContent:
  bytes = storage.get(content.hash)
  thumbnail = resize(bytes, 200, 200)
  thumb_hash = storage.put(thumbnail)
  SPAWN thumb: ImageContent { hash = thumb_hash, size = ..., width = 200, height = 200 }
  LINK has_thumbnail(content, thumb)
```

---

# Part VII: Layer 0 Extensions

## 7.1 Scalar Types

Add `Hash` to existing scalar types:

```
ScalarType = "String" | "Int" | "Float" | "Bool" | "Timestamp" | "Bytes" | "Hash"
```

That's it. No other kernel changes are required.

The `Hash` type:
- Fixed size (32 bytes for SHA-256)
- Comparable, hashable, indexable
- Stored inline in columnar storage
- No special semantics — client decides what it references

---

# Part VIII: Summary

## 8.1 Key Concepts

| Concept | Definition |
|---------|------------|
| **Control Plane** | Graph: structure, relationships, metadata, hashes |
| **Data Plane** | Storage: bytes, blobs, streams |
| **Content Node** | Conventional node pattern with `hash` attribute referencing external bytes |
| **Hash** | Fixed-size cryptographic identifier; opaque to kernel |
| **Content-Addressing** | Bytes identified by their hash |
| **Delegation** | Explicit transfer of key authority via graph edges |

## 8.2 Core Invariants

| Invariant | Meaning |
|-----------|---------|
| **Kernel never touches bytes** | Graph stores hashes, clients fetch content |
| **Hash is just a type** | No special kernel semantics; client decides meaning |
| **Trust is explicit** | Delegation visible as graph structure |
| **Encryption is optional** | Determined by presence of key edges |
| **Chunking is transparent** | Storage-layer concern, invisible to graph |

## 8.3 Components

| Component | Responsibility |
|-----------|----------------|
| **Kernel** | Stores Hash values as opaque 32-byte scalars |
| **StorageBackend node** | Graph structure holding backend config (type, endpoint, bucket, etc.) |
| **Client** | Resolves `stored_in` edges, connects to backend, fetches bytes |
| **Storage Backend (impl)** | Stores and retrieves bytes by hash (S3, IPFS, filesystem, etc.) |
| **GC Agent** | Cleans orphaned content |
| **Key Distribution Agent** | Manages encryption key distribution |

## 8.4 What's Added to MEW

| Addition | Description |
|----------|-------------|
| `Hash` scalar type | Fixed-size (32 bytes) cryptographic hash, comparable and indexable |

That's it. One scalar type.

Everything else — storage backends, chunking, encryption, key distribution, GC, the `Content` node pattern — is client-side implementation using standard MEW primitives (nodes, edges, subscriptions, agents).

---

*End of MEW Storage Specification*