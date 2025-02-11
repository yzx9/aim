# Copyright 2025 Zexin Yuan
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


from aiohttp import web

from aim.application import Application
from aim.domain.user.users import Users

__all__ = ["SessionHandler"]


class SessionHandler:
    def __init__(self, application: Application, users: Users):
        super().__init__()
        self._application = application
        self._users = users

    async def post(self, request: web.Request) -> web.Response:
        data = await request.json()
        userid = data.get("user_id")
        password = data.get("password")
        if not userid or not password:
            raise web.HTTPBadRequest(reason="Missing user_id or password")

        session = await self._application.session.login_by_password(userid, password)
        return web.json_response(
            {
                "access_token": session.access_token,
                "expire_at": session.access_payload.expire_at,
                "refresh_token": session.refresh_token,
            }
        )

    async def get(self, request: web.Request) -> web.Response:
        session = self._application.session.login_by_access_token(
            request.headers.get("Authorization")
        )
        return web.json_response("Hello, " + session.access_payload.username)
