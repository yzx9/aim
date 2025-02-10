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
import enum
import hashlib
from typing import Protocol

from aim.util import entity

__all__ = ["User", "UserRepository"]


class _PasswordTypes(enum.StrEnum):
    MD5 = "md5"
    NONE = "none"


@dataclasses.dataclass
class UserData:
    id: int
    name: str
    password_type: str
    password: str


class UserRepository(Protocol):
    async def save(self, user: UserData, /) -> None: ...
    async def find(self, id: int) -> UserData | None: ...


@entity
class User(UserData):
    def __init__(self, data: UserData, *, repository: UserRepository) -> None:
        super().__init__(**dataclasses.asdict(data))
        self._repository = repository

    # -----------------------
    #        Password
    # -----------------------
    # TODO: add salt
    # TODO: use ECC to encrypt the password

    def validate_password(self, password: str) -> bool:
        """Validates the given password against the stored password hash."""
        match self.password_type:
            case _PasswordTypes.MD5:
                hashed_password = hashlib.md5(password.encode("utf-8")).hexdigest()
                return hashed_password == self.password

            case _PasswordTypes.NONE:
                return False

            case _:
                raise ValueError(f"Unsupported password type: {self.password_type}")

    async def update_password(self, old_password: str, new_password: str) -> bool:
        """Updates the password if the old password is correct."""
        if not self.validate_password(old_password):
            return False

        self.password_type = _PasswordTypes.MD5
        self.password = hashlib.md5(new_password.encode("utf-8")).hexdigest()
        await self._save()
        return True

    async def reset_password(self, new_password: str) -> None:
        """Resets the password to a new password."""
        self.password_type = _PasswordTypes.MD5
        self.password = hashlib.md5(new_password.encode("utf-8")).hexdigest()
        await self._save()

    # -----------------------
    #         Misc
    # -----------------------

    async def _save(self, **kwargs) -> None:
        await self._repository.save(self, **kwargs)
