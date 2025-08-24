### Frontend Plan (Flutter)

Standards:
- State: riverpod; BLoC for complex flows
- HTTP: dio; JSON via json_serializable
- Models: generated from OpenAPI

Milestone 1: App skeleton
- Project structure (feature folders), theme, routing, error screens
- Env config and API base URL

Milestone 2: Auth & Profile
- Screens: login, signup, profile view/edit
- Secure storage for tokens; refresh flow
- Integration tests for happy-path

Milestone 3: Product & Rental flows
- Product list/detail, category filter, search
- Create rental, view bookings, cancel

Milestone 4: Messaging, Reviews, Subscription (MVP)
- In-app messaging threads/messages
- Post review and show ratings
- Subscription plan list and purchase (mock)

Quality:
- Widget golden tests; bloc/unit tests
- Lint: flutter analyze; format

Module breakdown:
- auth: login/signup/refresh, profile
- product: list/detail/categories/tags
- rental: availability/booking/list/cancel
- messaging: threads/messages (basic)
- reviews: list/create
- subscription: plans/list (basic)

API client generation:
- Generate Dart models/clients from OpenAPI on build (scripted task).
- Ensure response envelope mapping `{ data, error }`.


