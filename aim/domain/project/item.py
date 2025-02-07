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


from typing import Protocol

from aim.domain.project.field import FieldKind, FieldValue
from aim.util import Entity

__all__ = ["Item", "ItemValue", "ItemRepository"]


class ItemRepository(Protocol):
    async def save(self, value: "Item", /) -> None: ...
    async def delete(self, id: int, /) -> None: ...
    async def list_by_project(
        self, project_id: int, offset: int, limit: int
    ) -> list["Item"]: ...


class ItemValue:
    field_id: int
    kind: FieldKind
    value: FieldValue


class Item(Entity[int]):
    def __init__(
        self,
        id: int,
        project_id: int,
        values: list[ItemValue],
        *,
        repository: ItemRepository,
    ):
        super().__init__(id)
        self.project_id = project_id
        self.values = values
        self._repository = repository

    async def delete(self):
        """Delete the item."""
        await self._repository.delete(self.id)
