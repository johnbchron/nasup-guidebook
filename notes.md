Authorization header is a JWT:

```
Authorization: JWT eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjExMDE5Njc0LCJleHAiOjE3NzI3NzE2MjQsIjJmYSI6ZmFsc2V9.KhlgUohVfRGaWVbJrycP1cKEhCqyJxB-mMe9tEUx8o0
```

There's a `CurrentUser` cookie that contains a bunch of user data, including the
JWT and expiration time.

```json
{
  "user": {
    "id": 11019674,
    "is_staff": false,
    "is_accounting_team_member": false,
    "is_sales_ops": false,
    "is_super_staff": false,
    "two_factor_auth_enabled": false,
    "two_factor_setup_complete": false,
    "user_accepted_eula": null
  },
  "jwt": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjExMDE5Njc0LCJleHAiOjE3NzI3NzE2MjQsIjJmYSI6ZmFsc2V9.KhlgUohVfRGaWVbJrycP1cKEhCqyJxB-mMe9tEUx8o0",
  "session_expiration": "2026-03-06T04:33:44.788101+0000",
  "jwt_two_factor_verified": false
}
```

Importantly, to add a session, the request was:

```
POST https://builder.guidebook.com/api/sessions/
```

It's shaping up to look like a nice API.

This was the JSON body for the `POST`:

```json
{
  "name": {
    "en-US": "Test Session"
  },
  "locations": [],
  "schedule_tracks": [],
  "description_html": {
    "en-US": "<p>Test Session</p>"
  },
  "add_to_schedule": true,
  "image": null,
  "start_time": "2025-12-12T19:00:00.000Z",
  "end_time": "2025-12-12T20:00:00.000Z",
  "all_day": false,
  "guide": 205059,
  "allow_rating": true
}
```

Just looking at this, the `guide` is definitely an ID for the guide that this
session is for. It might be possible to set this as a constant for this project.

`locations` and `schedule_tracks` will probably also be ID lists.

I want to see what updating a session is like.

After I created a location, I updated the session to include it and this is what
the request looked like:

```
PATCH https://builder.guidebook.com/api/sessions/32179539/
```

```json
{
  "name": {
    "en-US": "Test Session"
  },
  "start_time": "2025-12-12T19:00:00.000000+0000",
  "end_time": "2025-12-12T20:00:00.000000+0000",
  "locations": [
    5164888
  ],
  "schedule_tracks": [],
  "description_html": {
    "en-US": "<p>Test Session</p>"
  },
  "add_to_schedule": true,
  "image": null,
  "all_day": false
}
```

Awesome. So all but `guide` and `allow_rating`. Maybe these can't be changed?

What about deleting?

```
DELETE https://builder.guidebook.com/api/sessions/32179539/
```

With no request body. This is shaping up to be a very well-behaved API.

Of course now is the time that I decide to look for an [API reference](https://developer.guidebook.com/#introduction) :facepalm:
