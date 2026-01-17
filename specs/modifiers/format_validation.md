---
spec: format_validation
version: "1.0"
status: stable
category: modifier
requires: []
priority: specialized
---

# Spec: Format Validation

## Overview

The `[format: name]` modifier validates string attributes against built-in format patterns. Provides common validations (email, URL, UUID) without writing regex patterns.

**Why needed:** Email, URL, and UUID validation are ubiquitous. Built-in formats are more readable than regex, optimized for performance, and can be used in query-time filtering.

---

## Syntax

### Grammar
```ebnf
AttributeModifier = ... | FormatModifier

FormatModifier = "format:" FormatName

FormatName = "email" | "url" | "uuid" | "slug" | "phone"
           | "iso_date" | "iso_datetime" | "ipv4" | "ipv6"
```

### Keywords

| Keyword | Context |
|---------|---------|
| `format` | Attribute modifier |

### Examples
```
node Person {
  email: String [format: email],
  website: String? [format: url],
  id: String [format: uuid]
}

node Server {
  hostname: String [format: slug],
  ip_address: String [format: ipv4]
}
```

---

## Semantics

### Built-in Formats

| Format | Description | Example |
|--------|-------------|---------|
| `email` | Email address | `user@example.com` |
| `url` | HTTP/HTTPS URL | `https://example.com/path` |
| `uuid` | UUID v4 format | `550e8400-e29b-41d4-a716-446655440000` |
| `slug` | URL-safe identifier | `my-blog-post-123` |
| `phone` | E.164 phone number | `+14155551234` |
| `iso_date` | ISO 8601 date | `2024-01-15` |
| `iso_datetime` | ISO 8601 datetime | `2024-01-15T10:30:00Z` |
| `ipv4` | IPv4 address | `192.168.1.1` |
| `ipv6` | IPv6 address | `2001:0db8::1` |

### Validation

Format validation occurs at SPAWN and SET time:
```
node Person {
  email: String [format: email]
}

SPAWN p: Person { email = "invalid" }
-- ERROR: 'invalid' is not a valid email format
```

### Null Handling

Format validation is skipped for null values:
```
node Person {
  website: String? [format: url]
}

SPAWN p: Person { website = null }  -- OK
```

### Query-Time Functions

Each format has a corresponding validation function for queries:
```
MATCH p: Person
WHERE is_email(p.contact_info)
RETURN p
```

| Format | Function |
|--------|----------|
| `email` | `is_email(s)` |
| `url` | `is_url(s)` |
| `uuid` | `is_uuid(s)` |
| `slug` | `is_slug(s)` |
| `phone` | `is_phone(s)` |
| `iso_date` | `is_iso_date(s)` |
| `iso_datetime` | `is_iso_datetime(s)` |
| `ipv4` | `is_ipv4(s)` |
| `ipv6` | `is_ipv6(s)` |

### Compilation to Constraint
```
email: String [format: email]
```

Compiles to:
```
constraint <type>_email_format:
  x: <Type> WHERE x.email != null
  => is_email(x.email)
```

---

## Layer 0

None. Format validation compiles to constraints using built-in functions.

---

## Compilation
```
node Person {
  email: String [required, format: email]
}
```

Compiles to:
```
_AttributeDef node:
  name: "email"
  scalar_type: "String"
  required: true

_ConstraintDef node:
  name: "Person_email_format"
  hard: true
  message: "Invalid email format"

-- Pattern: x: Person WHERE x.email != null
-- Condition: is_email(x.email)
```

---

## Examples

### User Registration
```
ontology Users {
  node User {
    username: String [required, format: slug, unique],
    email: String [required, format: email, unique],
    phone: String? [format: phone],
    website: String? [format: url]
  }
}
```

### API Configuration
```
ontology Infrastructure {
  node Service {
    name: String [required, format: slug],
    endpoint: String [required, format: url]
  }
  
  node Server {
    hostname: String [required, format: slug],
    ipv4: String? [format: ipv4],
    ipv6: String? [format: ipv6]
  }
  
  constraint server_has_ip:
    s: Server
    => s.ipv4 != null OR s.ipv6 != null
}
```

### External IDs
```
ontology Integration {
  node ExternalEntity {
    external_id: String [required, format: uuid, unique],
    source: String [required]
  }
}
```

### Query Filtering
```
-- Find users with potentially invalid emails (data cleanup)
MATCH u: User
WHERE NOT is_email(u.email)
RETURN u

-- Find servers without valid IPs
MATCH s: Server
WHERE NOT is_ipv4(s.ip) AND NOT is_ipv6(s.ip)
RETURN s
```

---

## Errors

| Condition | Message |
|-----------|---------|
| Invalid format | `"'value' is not a valid <format> format"` |
| Unknown format name | `"Unknown format 'xyz'"` |
| Format on non-string | `"[format] only valid for String attributes"` |

---

*End of Spec: Format Validation*