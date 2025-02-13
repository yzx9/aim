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

import argon2

from aim.domain.user.config import Config
from aim.domain.user.session import Session
from aim.util import entity

__all__ = ["User", "UserRepository"]


class _PasswordTypes(enum.StrEnum):
    NONE = "none"
    MD5 = "md5"
    ARGON2 = "argon2"


@dataclasses.dataclass
class UserData:
    id: int
    name: str
    password_type: str
    password_hash: str


class UserRepository(Protocol):
    async def save(self, user: UserData, /) -> None: ...
    async def find(self, id: int) -> UserData | None: ...


@entity
class User(UserData):
    def __init__(
        self, data: UserData, *, repository: UserRepository, config: Config
    ) -> None:
        super().__init__(**dataclasses.asdict(data))
        self._repository = repository
        self._config = config

    async def login(self, password: str) -> Session | None:
        """Login with the given password."""
        valid, need_rehash = self._validate_password(password)
        if not valid:
            return None

        if need_rehash:
            await self.reset_password(password)  # Rehash the password if needed

        return Session.new(self, config=self._config)

    # -----------------------
    #        Password
    # -----------------------

    async def update_password(self, old_password: str, new_password: str) -> bool:
        """Updates the password if the old password is correct."""
        valid, _ = self._validate_password(old_password)
        if not valid:
            return False

        self.password_type, self.password_hash = self._hash_password(new_password)
        await self._save()
        return True

    async def reset_password(self, new_password: str) -> None:
        """Resets the password to a new password."""
        self.password_type, self.password_hash = self._hash_password(new_password)
        await self._save()

    def _validate_password(self, password: str) -> tuple[bool, bool]:
        """Validates the given password against the stored password hash."""
        match self.password_type:
            case _PasswordTypes.MD5:
                hash = hashlib.md5(password.encode("utf-8")).hexdigest()
                valid = hash == self.password_hash
                need_rehash = valid  # Always rehash MD5 passwords
                return valid, need_rehash

            case _PasswordTypes.ARGON2:
                ph = argon2.PasswordHasher()
                try:
                    ph.verify(self.password_hash, password)
                    need_rehash = ph.check_needs_rehash(self.password_hash)
                    return True, need_rehash
                except argon2.exceptions.VerifyMismatchError:
                    return False, False

            case _PasswordTypes.NONE:  # No password set
                return False, False

            case _:
                raise ValueError(f"Unsupported password type: {self.password_type}")

    def _hash_password(self, password: str) -> tuple[_PasswordTypes, str]:
        """Hashes the given password."""
        ph = argon2.PasswordHasher()
        hash = ph.hash(password)
        return _PasswordTypes.ARGON2, hash

    # -----------------------
    #         Misc
    # -----------------------

    async def _save(self, **kwargs) -> None:
        await self._repository.save(self, **kwargs)
