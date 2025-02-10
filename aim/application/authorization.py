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

import dataclasses
import inspect
import time
from typing import (
    Any,
    Callable,
    Concatenate,
    Optional,
    ParamSpec,
    Self,
    TypeVar,
    overload,
)

import jwt

from aim.application.exceptions import (
    InvalidAuthorizationCredentials,
    InvalidAuthorizationToken,
    MissingAuthorizationToken,
)
from aim.domain.user import User
from aim.domain.user.users import Users

__all__ = ["make_authorization", "AccessTokenPayload", "SessionApplication"]


@dataclasses.dataclass
class AccessTokenPayload:
    user_id: int
    username: str
    expire_at: int

    def _to_claims(self) -> dict[str, Any]:
        return {"uid": self.user_id, "uname": self.username, "exp": self.expire_at}

    @classmethod
    def _form_claims(cls, claims: dict[str, Any]) -> Self:
        return cls(
            user_id=claims["uid"], username=claims["uname"], expire_at=claims["exp"]
        )


@dataclasses.dataclass
class RefershTokenPayload:
    user_id: int
    expire_at: int

    def _to_claims(self) -> dict[str, Any]:
        return {"uid": self.user_id, "exp": self.expire_at}

    @classmethod
    def _form_claims(cls, claims: dict[str, Any]) -> Self:
        return cls(user_id=claims["uid"], expire_at=claims["exp"])


def make_authorization(users: Users, jwt_secret: str):
    application = SessionApplication(users, jwt_secret)
    middleware = make_authorization_middleware(jwt_secret)
    return application, middleware


@dataclasses.dataclass
class Session:
    access_payload: AccessTokenPayload
    access_token: str
    refresh_token: Optional[str]


class SessionApplication:
    def __init__(
        self,
        users: Users,
        jwt_secret: str,
        exp_access_token: int = 5 * 60,  # 5 min
        exp_refresh_token: int = 30 * 24 * 60 * 60,  # 30 days
    ) -> None:
        super().__init__()
        self.users = users
        self._jwt_secret = jwt_secret
        self._exp_access_token = exp_access_token
        self._exp_refresh_token = exp_refresh_token

    async def new_session(self, user_id: int, password: str) -> Session:
        user = await self.users.find(user_id)
        if user is None or not user.validate_password(password):
            raise InvalidAuthorizationCredentials()

        access_token, access_payload = self._generate_access_token(user)
        refresh_token = self._generate_refresh_token(user)
        return Session(
            access_payload=access_payload,
            access_token=access_token,
            refresh_token=refresh_token,
        )

    async def try_refresh_access_token(self) -> bool:
        """Try to refresh the access token if it is about to expire."""
        threshold = 0.2
        need_refresh = (
            self.access_payload.expire_at - int(time.time())
            < threshold * self._exp_refresh_token
        )
        if not need_refresh:
            return False

        self.access_token, self.access_payload = self._generate_access_token(
            self.access_payload
        )
        return True

    async def refresh_access_token(self, refresh_token: str) -> bool:
        """Refresh the access token using the refresh token."""
        claims = jwt.decode(refresh_token, self._jwt_secret, algorithms=["HS256"])
        payload = RefershTokenPayload._form_claims(claims)
        if payload.user_id != payload.user_id:
            return False

        user = await self.users.find(payload.user_id)
        if user is None:
            raise InvalidAuthorizationToken()

        self.access_token, self.access_payload = self._generate_access_token(user)
        return True

    def _generate_access_token(
        self, user: User | AccessTokenPayload
    ) -> tuple[str, AccessTokenPayload]:
        expire_at = int(time.time()) + self._exp_access_token
        if isinstance(user, User):
            user_id = user.id
            username = user.name
        else:
            user_id = user.user_id
            username = user.username

        payload = AccessTokenPayload(
            user_id=user_id, username=username, expire_at=expire_at
        )
        token = jwt.encode(payload._to_claims(), self._jwt_secret)
        return token, payload

    def _generate_refresh_token(self, user: User) -> str:
        expire_at = int(time.time()) + self._exp_access_token
        payload = RefershTokenPayload(user_id=user.id, expire_at=expire_at)
        token = jwt.encode(payload._to_claims(), self._jwt_secret)
        return token


P = ParamSpec("P")
R = TypeVar("R")


def make_authorization_middleware(jwt_secret: str):
    @overload
    def required(
        handler: Callable[Concatenate[AccessTokenPayload | Session, P], R],
    ) -> Callable[Concatenate[str | None, P], R]: ...

    @overload
    def required(
        handler: Callable[P, R],
    ) -> Callable[Concatenate[str | None, P], R]: ...

    def required(
        handler: Callable[Concatenate[AccessTokenPayload | Session, P], R]
        | Callable[P, R],
    ) -> Callable[Concatenate[str | None, P], R]:
        """Authentication middleware as a function wrapper."""
        need_param = None
        sig = inspect.signature(handler)
        if len(sig.parameters):
            first_param_name = next(iter(sig.parameters))
            first_param_type = sig.parameters[first_param_name].annotation
            if first_param_type is AccessTokenPayload:
                need_param = "AccessTokenPayload"
            elif first_param_type is Session:
                need_param = "Session"

        def decorator(token: str | None, *args: P.args, **kwargs: P.kwargs) -> R:
            """Wrapper function to extract and validate the token."""
            # Assuming the first argument is the token string
            if token is None or not isinstance(token, str):
                raise MissingAuthorizationToken()

            try:
                claims = jwt.decode(token, jwt_secret, algorithms=["HS256"])
                payload = AccessTokenPayload._form_claims(claims)
            except Exception:
                raise InvalidAuthorizationToken()

            # Call the actual handler with the parsed payload
            match need_param:
                case "AccessTokenPayload":
                    return handler(payload, *args, **kwargs)  # type: ignore
                case "Session":
                    return handler(*args, **kwargs)  # type: ignore
                case _:
                    return handler(*args, **kwargs)  # type: ignore

        return decorator

    return required
