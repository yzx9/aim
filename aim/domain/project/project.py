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

from aim.domain.project.field import Field, FieldRepository
from aim.domain.project.item import Item, ItemData, ItemRepository, ItemValue
from aim.util import entity

__all__ = ["Project", "ProjectRepository"]


@dataclasses.dataclass
class ProjectData:
    id: int
    organization_id: int
    name: str


class ProjectRepository(Protocol):
    async def save(self, data: ProjectData) -> None: ...
    async def delete(self, id: int) -> ProjectData | None: ...
    async def find(self, id: int) -> ProjectData | None: ...


@entity
class Project(ProjectData):
    def __init__(
        self,
        data: ProjectData,
        *,
        repo_project: ProjectRepository,
        repo_item: ItemRepository,
        repo_field: FieldRepository,
    ):
        super().__init__(**dataclasses.asdict(data))
        self._repo_project = repo_project
        self._repo_item = repo_item
        self._repo_field = repo_field

    async def save(self, **kwargs):
        """Save the project."""
        await self._repo_project.save(self, **kwargs)

    async def delete(self):
        """Delete the project."""
        await self._repo_project.delete(self.id)

    async def get_fields(self) -> list[Field]:
        """Get all fields of the project."""
        data = await self._repo_field.list_by_project(self.id)
        return [Field(a, repository=self._repo_field) for a in data]

    async def list_items(self, offset: int, limit: int) -> list[Item]:
        """Get items of the project."""
        data = await self._repo_item.list_by_project(self.id, offset, limit)
        return [Item(a, repository=self._repo_item) for a in data]

    async def find_item(self, item_id: int) -> Item | None:
        data = await self._repo_item.find(item_id)
        if data is None or data.project_id != self.id:
            return None

        return Item(data, repository=self._repo_item)

    async def new_item(self, field_values: dict[int, str]) -> Item:
        """Create a new item."""
        fields = await self.get_fields()
        values = [
            ItemValue(
                field_id=a.id,
                kind=a.kind,
                value=field_values.get(a.id, a.default_value),
            )
            for a in fields
        ]

        data = ItemData(id=self.id, project_id=self.id, values=values)
        entity = Item(data, repository=self._repo_item)
        await entity._save()
        return entity
