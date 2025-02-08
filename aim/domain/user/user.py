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

from aim.util import entity

__all__ = ["User", "UserRepository"]


@dataclasses.dataclass
class UserData:
    id: int
    name: str


class UserRepository(Protocol):
    async def save(self, user: UserData, /) -> None: ...
    async def find(self, id: int) -> UserData | None: ...


@entity
class User(UserData):
    def __init__(self, data: UserData, *, repository: UserRepository) -> None:
        super().__init__(**dataclasses.asdict(data))
        self._repository = repository

    async def save(self, **kwargs) -> None:
        await self._repository.save(self, **kwargs)
