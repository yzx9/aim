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


from typing import Self

from aiohttp import web

from aim.application import Application
from aim.domain.user import Session, Users
from aim.interface.web.util import AuthedRequestModel, RequestModel, ResponseModel

__all__ = ["SessionHandler"]


class UserPassword(RequestModel):
    user_id: int
    password: str


class TokensResp(ResponseModel):
    access_token: str
    expire_at: int
    refresh_token: str | None

    @classmethod
    def from_session(cls, session: Session) -> Self:
        return cls(
            access_token=session.access_token,
            expire_at=session.access_payload.expire_at,
            refresh_token=session.refresh_token,
        )


class MessageResp(ResponseModel):
    message: str


class SessionHandler:
    def __init__(self, application: Application, users: Users):
        super().__init__()
        self._application = application
        self._session = application.session
        self._users = users

    async def post(self, request: web.Request) -> web.Response:
        req = await UserPassword.from_request(request)
        session = await self._session.login_by_password(req.user_id, req.password)
        return TokensResp.from_session(session).json_response()

    async def post_current(self, request: web.Request) -> web.Response:
        model = await AuthedRequestModel.from_request(request)
        session = await self._session.try_refresh_access_token(model.auth_token)
        return TokensResp.from_session(session).json_response()

    async def get_current(self, request: web.Request) -> web.Response:
        model = await AuthedRequestModel.from_request(request)
        session = self._session.login_by_access_token(model.auth_token)
        message = f"Hello, {session.access_payload.username}!"
        return MessageResp(message=message).json_response()

    async def del_current(self, request: web.Request) -> web.Response:
        model = await AuthedRequestModel.from_request(request)
        self._session.logout(model.auth_token)
        return MessageResp(message="Bye!").json_response()
