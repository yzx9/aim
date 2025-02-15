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
from datetime import datetime
from typing import Protocol

from aim.util import entity

__all__ = ["Field", "FieldRepository"]


class FieldKind(enum.StrEnum):
    """Value object"""

    NUMBER = "number"
    BOOLEAN = "boolean"
    DATETIME = "datetime"
    ENUM = "enum"
    STRING = "string"


@dataclasses.dataclass
class FieldEnumData:
    id: str
    value: str
    sort: int


class FieldEnumRepository(Protocol):
    async def save(self, field: FieldEnumData, /) -> None: ...
    async def delete(self, id: int) -> FieldEnumData | None: ...
    async def list_by_item(self, item_id: int, /) -> list[FieldEnumData]: ...


class FieldEnum(FieldEnumData):
    def __init__(self, data: FieldEnumData, *, repository: FieldEnumRepository):
        super().__init__(**dataclasses.asdict(data))
        self._repository = repository


FieldValue = float | bool | str | datetime


@dataclasses.dataclass
class FieldData:
    id: int
    project_id: int
    name: str
    kind: FieldKind
    default_value: FieldValue


class FieldRepository(Protocol):
    async def save(self, data: FieldData, /) -> None: ...
    async def delete(self, id: int) -> FieldData | None: ...
    async def list_by_project(self, project_id: int, /) -> list[FieldData]: ...


@entity
class Field(FieldData):
    def __init__(self, data: FieldData, *, repository: FieldRepository):
        super().__init__(**dataclasses.asdict(data))
        self._repository = repository

    async def _save(self, **kwargs):
        """Save the field."""
        # TODO: update all values
        await self._repository.save(self, **kwargs)

    async def _delete(self):
        """Delete the field."""
        # TODO: delete all values
        await self._repository.delete(self.id)
