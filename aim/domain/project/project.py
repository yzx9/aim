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

from aim.domain.project.field import Field, FieldRepository
from aim.domain.project.item import Item, ItemRepository
from aim.util import Entity

__all__ = ["Project", "ProjectRepository"]


class ProjectRepository(Protocol):
    async def save(self, project: "Project", /) -> None: ...
    async def delete(self, id: int) -> "Project | None": ...
    async def find(self, id: int) -> "Project | None": ...


class Project(Entity[int]):
    def __init__(
        self,
        id: int,
        organization_id: int,
        name: str,
        *,
        repo_project: ProjectRepository,
        repo_item: ItemRepository,
        repo_field: FieldRepository,
    ):
        super().__init__(id)
        self.organization_id = organization_id
        self.name = name
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
        return await self._repo_field.list_by_project(self.id)

    async def list(self, offset: int, limit: int) -> list[Item]:
        """Get items of the project."""
        return await self._repo_item.list_by_project(self.id, offset, limit)
