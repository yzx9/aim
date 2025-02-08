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
from datetime import datetime
from enum import Enum
from typing import Protocol

from aim.util import entity

__all__ = ["Field", "FieldRepository"]


class FieldKind(Enum):
    """Value object"""

    Number = "number"
    Datetime = "datetime"
    Enum = "enum"
    String = "string"


class EnumFieldRepository(Protocol):
    async def save(self, project: "Field", /) -> None: ...
    async def delete(self, id: int) -> "Field | None": ...
    async def list_by_project(self, project_id: int, /) -> "list[Field]": ...


@dataclasses.dataclass
class EnumField:
    id: str
    value: str
    sort: int


FieldValue = int | float | str | datetime


@dataclasses.dataclass
class FieldData:
    id: int
    project_id: int
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
