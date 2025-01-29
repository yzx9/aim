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

from abc import ABC, abstractmethod
from typing import Any, Generic, TypeVar

from aim.util.id_generator import IdGenerator

__all__ = ["Aggregate", "Entity", "ValueObject"]


class ValueObject:
    pass


T_id = TypeVar("T_id", bound=int | str)


class Entity(ABC, Generic[T_id]):
    def __init__(self, id: T_id):
        super().__init__()
        self.id = id

    @abstractmethod
    async def save(self) -> None: ...


T_aggregate_root = TypeVar("T_aggregate_root", bound=Entity[Any], covariant=True)


class Aggregate(ABC, Generic[T_aggregate_root, T_id]):
    def __init__(self, *, id_generator: IdGenerator[T_id]) -> None:
        super().__init__()
        self._id_generator = id_generator

    @abstractmethod
    async def find(self, id: T_id) -> T_aggregate_root | None: ...
