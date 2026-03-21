# API

Base URL: `/api`

All authenticated endpoints require the token issued by `AdminSeeder` as a Bearer token:

```
Authorization: Bearer <token>
```

---

### Posts

#### `GET /api/posts`

Returns a paginated list of published posts. Drafts (`published_at = null`) are excluded.

**Auth:** No

**Response `200`**
```json
{
  "data": [
    {
      "title": "My Post",
      "slug": "my-post",
      "excerpt": "A short summary.",
      "published_at": "2026-03-19T00:00:00.000000Z"
    }
  ],
  "current_page": 1,
  "last_page": 1,
  ...
}
```

---

#### `GET /api/posts/{slug}`

Returns a single published post by slug.

**Auth:** No

**Response `200`**
```json
{
  "id": 1,
  "title": "My Post",
  "slug": "my-post",
  "body": "Full post content...",
  "excerpt": "A short summary.",
  "published_at": "2026-03-19T00:00:00.000000Z",
  "created_at": "2026-03-19T00:00:00.000000Z",
  "updated_at": "2026-03-19T00:00:00.000000Z"
}
```

Returns `404` if the post does not exist or is not published.

---

#### `POST /api/posts`

Creates a new post. The slug is auto-generated from the title. Posts are created as drafts unless `published_at` is provided.

**Auth:** Yes

**Body**
```json
{
  "title": "My Post",
  "body": "Full post content...",
  "excerpt": "A short summary.",
  "published_at": "2026-03-19T00:00:00Z"
}
```

| Field | Type | Required |
|-------|------|----------|
| `title` | string (max 255) | Yes |
| `body` | string | Yes |
| `excerpt` | string | No |
| `published_at` | datetime | No |

**Response `201`** — the created post object.

---

#### `PUT /api/posts/{slug}`

Updates a post. Only the fields provided are changed. If `title` is updated, the slug is regenerated.

**Auth:** Yes

**Body** — any subset of the fields from `POST /api/posts`.

**Response `200`** — the updated post object.

---

#### `DELETE /api/posts/{slug}`

Deletes a post permanently.

**Auth:** Yes

**Response `204`** — no content.

---

#### `POST /api/posts/{slug}/publish`

Sets `published_at` to the current timestamp, making the post publicly visible.

**Auth:** Yes

**Response `200`** — the updated post object.

---

#### `POST /api/posts/{slug}/unpublish`

Sets `published_at` to `null`, reverting the post to a draft.

**Auth:** Yes

**Response `200`** — the updated post object.

---

### Projects

#### `DELETE /api/projects/{id}`

Deletes a project permanently.

**Auth:** Yes

**Response `204`** — no content.

---

#### `POST /api/projects/{id}/publish`

Sets the project's `published_at` to now, making it appear on the portfolio page.

**Auth:** Yes

**Response `200`** — the updated project object.

---

#### `POST /api/projects/{id}/unpublish`

Sets the project's `published_at` to `null`, hiding it from the portfolio page.

**Auth:** Yes

**Response `200`** — the updated project object.

---

## Data Model

### Post

| Column | Type | Notes |
|--------|------|-------|
| `id` | integer | Primary key |
| `title` | string | |
| `slug` | string | Unique, auto-generated from title |
| `body` | longtext | |
| `excerpt` | text | Nullable |
| `published_at` | timestamp | Nullable — null means draft |
| `created_at` | timestamp | |
| `updated_at` | timestamp | |

### Project

| Column | Type | Notes |
|--------|------|-------|
| `id` | integer | Primary key |
| `name` | string | |
| `description` | text | |
| `github_url` | string | Nullable |
| `project_url` | string | Nullable |
| `tech_stack` | string | Comma-separated |
| `published_at` | timestamp | Nullable — null means hidden |
| `created_at` | timestamp | |
| `updated_at` | timestamp | |

---

## Authentication

This is a single-user system. There is no registration or login endpoint. The API token is issued once via `AdminSeeder` and used for all write operations. If the token is lost, generate a new one via Tinker:

```bash
php artisan tinker
>>> App\Models\User::first()->createToken('admin')->plainTextToken
```
