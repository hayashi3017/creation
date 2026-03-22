Authenticate a user

Validates the submitted credentials and returns a token payload.

The current implementation also sets an HttpOnly `token` cookie so the same authenticated session can be reused on later requests.
