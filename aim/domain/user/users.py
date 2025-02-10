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
from typing import Protocol

from aim.domain.user.user import User, UserData, UserRepository
from aim.util import IdGenerator, aggregate

__all__ = ["Users", "Repository"]


class Repository(Protocol):
    @property
    def users(self) -> UserRepository: ...


@aggregate
class Users:
    def __init__(self, *, repository: Repository, id_generator: IdGenerator[int]):
        super().__init__()
        self._repository = repository
        self._id_generator = id_generator

    async def new(self, name: str) -> User:
        """Create and save a new user.

        Returns
        -------
        Self
            A new project that has been persisted
        """
        id = self._id_generator.generate()
        data = UserData(id=id, name=name)
        user = User(data, repository=self._repository.users)
        await user._save()
        return user

    async def find(self, id: int) -> User | None:
        """Find an user by its ID.

        Parameters
        ----------
        id : int
            The ID of the user to find

        Returns
        -------
        User | None
            The found user, or None if not found
        """
        data = await self._repository.users.find(id)
        if data is None:
            return None

        return User(**dataclasses.asdict(data), repository=self._repository.users)
