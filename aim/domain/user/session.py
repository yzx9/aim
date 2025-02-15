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
import time
import uuid
from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any, Self, Type

import jwt

from aim.domain.user.config import Config
from aim.util import entity, value_object

if TYPE_CHECKING:
    from aim.domain.user.user import User
    from aim.domain.user.users import Users

__all__ = ["Session", "AccessTokenPayload", "RefershTokenPayload"]


class TokenPayload(ABC):
    @abstractmethod
    def _to_claims(self) -> dict[str, Any]: ...

    @classmethod
    @abstractmethod
    def _form_claims(cls, claims: dict[str, Any]) -> Self: ...


@value_object
@dataclasses.dataclass
class AccessTokenPayload(TokenPayload):
    session_id: str
    user_id: int
    username: str
    expire_at: int

    def _to_claims(self) -> dict[str, Any]:
        return {
            "sid": self.session_id,
            "uid": self.user_id,
            "uname": self.username,
            "exp": self.expire_at,
        }

    @classmethod
    def _form_claims(cls, claims: dict[str, Any]) -> Self:
        return cls(
            session_id=claims["sid"],
            user_id=claims["uid"],
            username=claims["uname"],
            expire_at=claims["exp"],
        )


@value_object
@dataclasses.dataclass
class RefershTokenPayload(TokenPayload):
    session_id: str
    user_id: int
    expire_at: int

    def _to_claims(self) -> dict[str, Any]:
        return {"sid": self.session_id, "uid": self.user_id, "exp": self.expire_at}

    @classmethod
    def _form_claims(cls, claims: dict[str, Any]) -> Self:
        return cls(
            session_id=claims["sid"], user_id=claims["uid"], expire_at=claims["exp"]
        )


@entity
class Session:
    def __init__(
        self,
        id: str,
        access_token: str,
        access_payload: AccessTokenPayload,
        refresh_token: str | None = None,
        *,
        config: Config,
    ) -> None:
        super().__init__()
        self.id = id
        self.access_payload = access_payload
        self.access_token = access_token
        self.refresh_token = refresh_token
        self._config = config

    async def try_refresh_access_token(self) -> bool:
        """Try to refresh the access token if it is about to expire."""
        threshold = 0.2
        need_refresh = (
            self.access_payload.expire_at - int(time.time())
            < threshold * self._config.exp_refresh_token
        )
        if not need_refresh:
            return False

        self.access_token, self.access_payload = _new_access_token(
            self._config, self.id, self.access_payload
        )
        return True

    @classmethod
    def new(cls, user: "User", *, config: Config) -> Self:
        id = str(uuid.uuid4())
        access_token, access_payload = _new_access_token(config, id, user)
        refresh_token = _new_refresh_token(config, id, user)
        return cls(id, access_token, access_payload, refresh_token, config=config)

    @classmethod
    def _from_access_token(cls, token: str, *, config: Config) -> Self | None:
        payload = _decode_token(config, AccessTokenPayload, token)
        if payload is None:
            return None

        return cls(payload.session_id, token, payload, config=config)

    @classmethod
    async def _from_refresh_token(
        cls, token: str, *, config: Config, users: "Users"
    ) -> Self | None:
        """Refresh the access token using the refresh token."""
        payload = _decode_token(config, RefershTokenPayload, token)
        if payload is None or payload.user_id != payload.user_id:
            return None

        user = await users.find(payload.user_id)
        if user is None:
            return None

        id = str(uuid.uuid4())
        access_token, access_payload = _new_access_token(config, id, user)
        return cls(id, access_token, access_payload, token, config=config)


def _new_access_token(
    config: Config, session_id: str, user: "User | AccessTokenPayload"
) -> tuple[str, AccessTokenPayload]:
    expire_at = int(time.time()) + config.exp_access_token
    if isinstance(user, AccessTokenPayload):
        user_id = user.user_id
        username = user.username
    else:
        user_id = user.id
        username = user.name

    payload = AccessTokenPayload(
        session_id=session_id,
        user_id=user_id,
        username=username,
        expire_at=expire_at,
    )
    token = jwt.encode(payload._to_claims(), config.jwt_secret)
    return token, payload


def _new_refresh_token(config: Config, session_id: str, user: "User") -> str:
    expire_at = int(time.time()) + config.exp_access_token
    payload = RefershTokenPayload(
        session_id=session_id, user_id=user.id, expire_at=expire_at
    )
    token = jwt.encode(payload._to_claims(), config.jwt_secret)
    return token


def _decode_token[T: TokenPayload](
    config: Config, payload: Type[T], token: str
) -> T | None:
    try:
        claims = jwt.decode(token, config.jwt_secret, algorithms=["HS256"])
    except jwt.ExpiredSignatureError:
        return None
    except jwt.InvalidTokenError:
        return None

    try:
        parsed = payload._form_claims(claims)
    except KeyError:
        return None

    return parsed
