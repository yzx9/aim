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


from typing import Protocol

from aim.application.authorization import AuthorizationMiddleware
from aim.application.exceptions import InvalidAuthorizationCredentials
from aim.domain.user import Session, Users

__all__ = ["make_session", "SessionApplication"]


class SessionApplication(Protocol):
    @staticmethod
    async def login_by_password(user_id: int, password: str) -> Session: ...

    @staticmethod
    def login_by_access_token(token: str | None, /) -> Session: ...

    @staticmethod
    def logout(token: str | None, /) -> None: ...

    @staticmethod
    async def try_refresh_access_token(token: str | None, /) -> Session: ...


def make_session(users: Users, auth: AuthorizationMiddleware) -> SessionApplication:
    class Application:
        @staticmethod
        async def login_by_password(user_id: int, password: str) -> Session:
            user = await users.find(user_id)
            if user is None:
                raise InvalidAuthorizationCredentials()

            session = await user.login(password)
            if session is None:
                raise InvalidAuthorizationCredentials()

            return session

        @auth.required
        @staticmethod
        def login_by_access_token(session: Session) -> Session:
            return session

        @auth.required
        @staticmethod
        def logout(session: Session) -> None:
            pass  # do nothing

        @auth.required
        @staticmethod
        async def try_refresh_access_token(session: Session) -> Session:
            await session.try_refresh_access_token()
            return session

    return Application()
