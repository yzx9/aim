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

from aim.domain.project.field import FieldKind, FieldValue
from aim.util import entity, value_object

__all__ = ["Item", "ItemValue", "ItemRepository"]


@value_object
@dataclasses.dataclass
class ItemValue:
    field_id: int
    kind: FieldKind
    value: FieldValue


@dataclasses.dataclass
class ItemData:
    id: int
    project_id: int
    values: list[ItemValue]


class ItemRepository(Protocol):
    async def find(self, id: int) -> ItemData: ...
    async def save(self, data: ItemData) -> None: ...
    async def delete(self, id: int) -> None: ...
    async def list_by_project(
        self, project_id: int, offset: int, limit: int
    ) -> list[ItemData]: ...


@entity
class Item(ItemData):
    def __init__(self, data: ItemData, *, repository: ItemRepository):
        super().__init__(**dataclasses.asdict(data))
        self._repository = repository

    async def update_values(self, *values: ItemValue):
        """Update the value of the item."""
        value_dict = {value.field_id: value for value in values}
        for v in self.values:
            if v.field_id in value_dict:
                v.value = value_dict[v.field_id].value  # only update value

        await self._save()

    async def delete(self):
        """Delete the item."""
        await self._repository.delete(self.id)

    async def _save(self) -> None:
        await self._repository.save(self)
